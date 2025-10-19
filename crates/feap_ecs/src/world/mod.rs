mod deferred_world;
mod error;
mod identifier;

pub use deferred_world::DeferredWorld;
pub use identifier::WorldId;

use self::error::*;
use crate::{
    change_detection::{MaybeLocation, Mut, MutUntyped, TicksMut},
    component::{
        CHECK_TICK_THRESHOLD, CheckChangeTicks, ComponentId, ComponentIds, Components,
        ComponentsRegistrator, Tick,
    },
    resource::Resource,
    schedule::{Schedule, Schedules},
    storage::{ResourceData, Storages},
    query::DebugCheckedUnwrap,
};
use core::{
    any::TypeId,
    cell::UnsafeCell,
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use feap_core::ptr::{OwningPtr, UnsafeCellDeref};
use feap_ecs::schedule::ScheduleLabel;
use feap_utils::debug_info::DebugName;

/// Variant of the [`World`] where resource and component accesses take `&self`, and the responsibility to avoid
/// aliasing violations are given to the caller instead of being checked at compile-time by rust's unique XOR shared rule.
///
#[derive(Copy, Clone)]
pub struct UnsafeWorldCell<'w> {
    ptr: *mut World,
    #[cfg(debug_assertions)]
    allows_mutable_access: bool,
    _marker: PhantomData<(&'w World, &'w UnsafeCell<World>)>,
}

unsafe impl Send for UnsafeWorldCell<'_> {}
unsafe impl Sync for UnsafeWorldCell<'_> {}

impl<'w> From<&'w mut World> for UnsafeWorldCell<'w> {
    fn from(value: &'w mut World) -> Self {
        value.as_unsafe_world_cell()
    }
}

impl<'w> From<&'w World> for UnsafeWorldCell<'w> {
    fn from(value: &'w World) -> Self {
        value.as_unsafe_world_cell_readonly()
    }
}

impl<'w> UnsafeWorldCell<'w> {
    /// Creates a [`UnsafeWorldCell`] that can be used to access everything immutably
    #[inline]
    pub(crate) fn new_readonly(world: &'w World) -> Self {
        Self {
            ptr: ptr::from_ref(world).cast_mut(),
            #[cfg(debug_assertions)]
            allows_mutable_access: false,
            _marker: PhantomData,
        }
    }

    /// Creates [`UnsafeWorldCell`] that can be used to access everything mutably
    #[inline]
    pub(crate) fn new_mutable(world: &'w mut World) -> Self {
        Self {
            ptr: ptr::from_mut(world),
            #[cfg(debug_assertions)]
            allows_mutable_access: true,
            _marker: PhantomData,
        }
    }

    #[cfg_attr(debug_assertions, inline(never), track_caller)]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub(crate) fn assert_allows_mutable_access(self) {
        #[cfg(debug_assertions)]
        debug_assert!(
            self.allows_mutable_access,
            "mutating world data via `World::as_unsafe_world_cell_readonly` is forbidden"
        );
    }

    /// Variant of [`UnsafeWorldCell::world`] solely used for implementing this type's methods
    /// It allows having an `&World` even with live mutable borrows of components and resources
    #[inline]
    unsafe fn unsafe_world(self) -> &'w World {
        unsafe { &*self.ptr }
    }

    /// Gets a reference to the [`World`] this [`UnsafeWorldCell`] belongs to
    /// This can be used for arbitrary read only access of world metadata
    #[inline]
    pub unsafe fn world_metadata(self) -> &'w World {
        unsafe { self.unsafe_world() }
    }

    /// Retrieves this world's [`Components`] collection
    #[inline]
    pub fn components(self) -> &'w Components {
        &unsafe { self.world_metadata() }.components
    }

    /// Provides unchecked access to the internal data stores of the [`World`]
    #[inline]
    pub unsafe fn storages(self) -> &'w Storages {
        &unsafe { self.unsafe_world() }.storages
    }

    /// Gets a mutable reference to the resource of the given type if it exists
    #[inline]
    pub unsafe fn get_resource_mut<R: Resource>(self) -> Option<Mut<'w, R>> {
        self.assert_allows_mutable_access();
        let component_id = self.components().get_valid_resource_id(TypeId::of::<R>())?;
        unsafe {
            self.get_resource_mut_by_id(component_id)
                .map(|ptr| ptr.with_type::<R>())
        }
    }

    /// Gets a pointer to the resource with the id [`ComponentId`] if it exists
    /// The returned pointer may be used to modify the resource, as long as the mutable borrow
    /// of the [`UnsafeWorldCell`] is still valid
    #[inline]
    pub unsafe fn get_resource_mut_by_id(
        self,
        component_id: ComponentId,
    ) -> Option<MutUntyped<'w>> {
        self.assert_allows_mutable_access();
        let (ptr, ticks, caller) = unsafe { self.storages() }
            .resources
            .get(component_id)?
            .get_with_ticks()?;

        let ticks = unsafe {
            TicksMut::from_tick_cells(ticks, self.last_change_tick(), self.change_tick())
        };

        Some(MutUntyped {
            value: unsafe { ptr.assert_unique() },
            ticks,
            changed_by: unsafe { caller.map(|caller| caller.deref_mut()) },
        })
    }

    /// Gets the current change tick of this world
    #[inline]
    pub fn change_tick(self) -> Tick {
        unsafe { self.world_metadata() }.read_change_tick()
    }

    /// Returns the [`Tick`] indicating the last time that [`World::clear_trackers`] was called
    #[inline]
    pub fn last_change_tick(self) -> Tick {
        unsafe { self.world_metadata() }.last_change_tick()
    }
}

/// Stores and exposes operations on entities, components, resources and their associated metadata
///
/// Each [`Entity`] has a set of unique components, based on their type
/// Entity components can be created, updated, removed, and queried using a given
///
pub struct World {
    id: WorldId,
    pub(crate) components: Components,
    pub(crate) component_ids: ComponentIds,
    pub(crate) storages: Storages,
    pub(crate) change_tick: AtomicU32,
    pub(crate) last_change_tick: Tick,
}

impl Default for World {
    fn default() -> Self {
        let mut world = Self {
            id: WorldId::new().expect("More worlds have been created than supported"),
            components: Components::default(),
            component_ids: ComponentIds::default(),
            storages: Storages::default(),
            change_tick: AtomicU32::new(1),
            last_change_tick: Tick::new(0),
        };
        world.bootstrap();
        world
    }
}

impl World {
    /// This performs initialization that _must_ happen for every [`World`] immediately upon creation
    #[inline]
    fn bootstrap(&mut self) {}

    /// Creates a new empty [`World`]
    #[inline]
    pub fn new() -> World {
        World::default()
    }

    /// Retrieves this [`World`]s unique ID
    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

    /// Creates a new [`UnsafeWorldCell`] view with complete read+write access
    #[inline]
    pub fn as_unsafe_world_cell(&mut self) -> UnsafeWorldCell<'_> {
        UnsafeWorldCell::new_mutable(self)
    }

    /// Creates a new [`UnsafeWorldCell`] view with only read access to everything.
    #[inline]
    pub fn as_unsafe_world_cell_readonly(&self) -> UnsafeWorldCell<'_> {
        UnsafeWorldCell::new_readonly(self)
    }

    /// Prepares a [`ComponentRegistrator`] for the world
    #[inline]
    pub fn components_registrator(&mut self) -> ComponentsRegistrator {
        unsafe { ComponentsRegistrator::new(&mut self.components, &mut self.component_ids) }
    }

    /// Initializes a new resource and returns the [`ComponentId`] created for it
    ///
    /// If the resource already exists, nothing happens
    #[inline]
    #[track_caller]
    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> ComponentId {
        let caller = MaybeLocation::caller();
        let component_id = self.components_registrator().register_resource::<R>();
        if self
            .storages
            .resources
            .get(component_id)
            .is_none_or(|data| !data.is_present())
        {
            let value = R::from_world(self);
            OwningPtr::make(value, |ptr| unsafe {
                self.insert_resource_by_id(component_id, ptr, caller);
            });
        }
        component_id
    }
    
    /// Gets a mutable reference to the resource of type `T` if it exists,
    /// otherwise initializes the resource by calling its [`FromWorld`] implementation
    #[track_caller]
    pub fn get_resource_or_init<R: Resource + FromWorld>(&mut self) -> Mut<'_, R> {
        let caller = MaybeLocation::caller();
        let change_tick = self.change_tick();
        let last_change_tick = self.last_change_tick();
        
        let component_id = self.components_registrator().register_resource::<R>();
        if self.storages
            .resources
            .get(component_id)
            .is_none_or(|data| !data.is_present()) 
        {
            let value = R::from_world(self);
            OwningPtr::make(value, |ptr| {
                unsafe { self.insert_resource_by_id(component_id, ptr, caller); }
            });
        }
        
        let data = unsafe {
          self.storages
              .resources
              .get_mut(component_id)
              .debug_checked_unwrap()
        };
        
        let data = unsafe {
            data.get_mut(last_change_tick, change_tick)
                .debug_checked_unwrap()
        };
        
        unsafe { data.with_type::<R>() }
    }

    /// Inserts a new resource with the given `value`. Will replace the value if it already exists
    #[inline]
    #[track_caller]
    pub unsafe fn insert_resource_by_id(
        &mut self,
        component_id: ComponentId,
        value: OwningPtr<'_>,
        caller: MaybeLocation,
    ) {
        let change_tick = self.change_tick();

        let resource = self.initialize_resource_internal(component_id);
        unsafe {
            resource.insert(value, change_tick, caller);
        }
    }

    #[inline]
    pub(crate) fn initialize_resource_internal(
        &mut self,
        component_id: ComponentId,
    ) -> &mut ResourceData<true> {
        self.flush_components();
        self.storages
            .resources
            .initialize_with(component_id, &self.components)
    }

    /// Gets a mutable reference to the resource of the given type
    /// Panics if the resource does not exist
    #[inline]
    #[track_caller]
    pub fn resource_mut<R: Resource>(&mut self) -> Mut<'_, R> {
        match self.get_resource_mut() {
            Some(x) => x,
            None => panic!(
                "Requested resource {} does not exist in the `World`.
                Did you forget to add it using `app.insert_resource` / `app.init_resource`?
                Resources are also implicitly added via `app.add_message`,
                and can be added by plugins.",
                DebugName::type_name::<R>()
            ),
        }
    }

    /// Gets a mutable reference to the resource of the given type if it exists
    #[inline]
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<Mut<'_, R>> {
        unsafe { self.as_unsafe_world_cell().get_resource_mut() }
    }

    /// Applies any queued component registration
    pub(crate) fn flush_components(&mut self) {
        self.components_registrator().apply_queued_registrations();
    }

    /// Runs the [`Schedule`] associated with the `label` a single time
    ///
    /// The [`Schedule`] is fetched from the [`Schedules`] resource of the world by its label,
    /// and system state is cached
    pub fn run_schedule(&mut self, label: impl ScheduleLabel) {
        self.schedule_scope(label, |world, sched| sched.run(world));
    }

    /// Temporarily removes the schedule associated with `label` from the world,
    /// runs user code, and finally re-adds the schedule
    ///
    /// The [`Schedule`] is fetched from the [`Schedules`] resource of the world by its label,
    /// and system state is cached
    pub fn schedule_scope<R>(
        &mut self,
        label: impl ScheduleLabel,
        f: impl FnOnce(&mut World, &mut Schedule) -> R,
    ) -> R {
        self.try_schedule_scope(label, f)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    fn try_schedule_scope<R>(
        &mut self,
        label: impl ScheduleLabel,
        f: impl FnOnce(&mut World, &mut Schedule) -> R,
    ) -> Result<R, TryRunScheduleError> {
        let label = label.intern();
        let Some(mut schedule) = self
            .get_resource_mut::<Schedules>()
            .and_then(|mut s| s.remove(label))
        else {
            return Err(TryRunScheduleError(label));
        };

        let value = f(self, &mut schedule);

        let old = self.resource_mut::<Schedules>().insert(schedule);
        if old.is_some() {
            log::warn!(
                "Schedule `{label:?}` was inserted during a call to `World::schedule_scope`: its value has been overwritten"
            );
        }

        Ok(value)
    }

    /// Reads the current change tick of this world
    #[inline]
    pub fn read_change_tick(&self) -> Tick {
        let tick = self.change_tick.load(Ordering::Acquire);
        Tick::new(tick)
    }

    /// Reads the current change tick of this world
    #[inline]
    pub fn change_tick(&mut self) -> Tick {
        let tick = *self.change_tick.get_mut();
        Tick::new(tick)
    }

    /// When called from within an exclusive system (a [`System`] that takes `&mut World` as its first
    /// parameter), this method returns the [`Tick`] indicating the last time the exclusive system was run.
    ///
    /// Otherwise, this returns the `Tick` indicating the last time that [`World::clear_trackers`] was called.
    #[inline]
    pub fn last_change_tick(&self) -> Tick {
        self.last_change_tick
    }

    /// Iterates all component change ticks and clamps any older than [`MAX_CHANGE_AGE`]
    /// This also triggers [`CheckChangeTicks`] observers and returns the same event here
    ///
    /// Calling this method prevents [`Tick`]s overflowing and thus prevents false positives when comparing them
    pub fn check_change_ticks(&mut self) -> Option<CheckChangeTicks> {
        let change_tick = self.change_tick();
        if change_tick.relative_to(self.last_change_tick).get() < CHECK_TICK_THRESHOLD {
            return None;
        }

        todo!()
    }
}

/// Creates an instance of the type this trait is implemented for
/// using data from the supplied [`World`]
///
/// This can be helpful for complex initialization or contgext-aware defaults
///
/// [`FromWorld`] is automatically implemented for any type implementing [`Default`]
/// and may also be derive for
/// - any struct whose fields all implement `FromWorld`
/// - any enum where one variant has the attribute `#[from_world]`
///
pub trait FromWorld {
    /// Creates `Self` using data from the given [`World`]
    fn from_world(world: &mut World) -> Self;
}

impl<T: Default> FromWorld for T {
    fn from_world(_world: &mut World) -> Self {
        T::default()
    }
}
