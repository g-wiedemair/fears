use crate::{
    change_detection::{MaybeLocation, MutUntyped, TicksMut},
    component::{ComponentId, Components, Tick, TickCells},
    storage::{blob_array::BlobArray, sparse_set::SparseSet},
};
use feap_core::ptr::{OwningPtr, Ptr, UnsafeCellDeref};
use feap_utils::debug_info::DebugName;
use core::{cell::UnsafeCell, panic::Location};
#[cfg(feature = "std")]
use std::thread::ThreadId;

/// The type-erased backing storage and metadata for a single resource within a [`World`]
/// If `SEND` is false, value of this type will panic if dropped from a different thread
pub struct ResourceData<const SEND: bool> {
    data: BlobArray,
    is_present: bool,
    added_ticks: UnsafeCell<Tick>,
    changed_ticks: UnsafeCell<Tick>,
    #[cfg_attr(
        not(feature = "std"), 
        expect(dead_code, reason = "currently only used with the std feature")
    )]
    type_name: DebugName,
    #[cfg(feature = "std")]
    origin_thread_id: Option<ThreadId>,
    changed_by: MaybeLocation<UnsafeCell<&'static Location<'static>>>,
}

impl<const SEND: bool> ResourceData<SEND> {
    /// The only row in the underlying `BlobArray`.
    const ROW: usize = 0;

    /// Validates the access to `!Send` resources is only done on the thread they were created
    #[inline]
    fn validate_access(&self) {
        if !SEND {
            #[cfg(feature = "std")]
            if self.origin_thread_id != Some(std::thread::current().id()) {
                panic!(
                "Attempted to access or drop non-send resource {} from thread {:?} on a thread {:?}. This is not allowed. Aborting.",
                self.type_name,
                self.origin_thread_id,
                std::thread::current().id()
                );
            }
        }
    }

    /// Returns true if the resource is populated
    #[inline]
    pub fn is_present(&self) -> bool {
        self.is_present
    }

    /// Inserts a value into the resource. If a value is already present it will
    /// be replaced.
    #[inline]
    pub(crate) unsafe fn insert(
        &mut self,
        value: OwningPtr<'_>,
        change_tick: Tick,
        caller: MaybeLocation,
    ) {
        if self.is_present() {
            todo!()
        } else {
            #[cfg(feature = "std")]
            if !SEND {
                self.origin_thread_id = Some(std::thread::current().id());
            }

            unsafe { self.data.initialize_unchecked(Self::ROW, value)};
            *self.added_ticks.deref_mut() = change_tick;
            self.is_present = true;
        }
        *self.changed_ticks.deref_mut() = change_tick;

        self.changed_by.as_ref().map(|changed_by| changed_by.deref_mut()).assign(caller);
    }
    
    /// Returns a mutable reference to the resource, it if exists
    pub(crate) fn get_mut(&mut self, last_run: Tick, this_run: Tick) -> Option<MutUntyped<'_>> {
        let (ptr, ticks, caller) = self.get_with_ticks()?;
        Some(MutUntyped {
          value: unsafe { ptr.assert_unique() },
            ticks: unsafe { TicksMut::from_tick_cells(ticks, last_run, this_run)},
            changed_by: unsafe { caller.map(|caller| caller.deref_mut())}
        })
    }

    /// Returns references to the resource and its change ticks, if it exists
    #[inline]
    pub(crate) fn get_with_ticks(
        &self
    ) -> Option<(
        Ptr<'_>,
        TickCells<'_>,
        MaybeLocation<&UnsafeCell<&'static Location<'static>>>,
    )> {
        self.is_present().then(|| {
            self.validate_access();
            (
                unsafe { self.data.get_unchecked(Self::ROW)},
                TickCells {
                    added: &self.added_ticks,
                    changed: &self.changed_ticks,
                },
                self.changed_by.as_ref(),
            )
        })
    }
}

/// The backing store for all [`Resource`]s stored in the [`World`]
#[derive(Default)]
pub struct Resources<const SEND: bool> {
    resources: SparseSet<ComponentId, ResourceData<SEND>>,
}

impl<const SEND: bool> Resources<SEND> {
    /// Fetches or initializes a new resource and returns back its underlying column
    pub(crate) fn initialize_with(
        &mut self,
        component_id: ComponentId,
        components: &Components,
    ) -> &mut ResourceData<SEND> {
        self.resources.get_or_insert_with(component_id, || {
            let component_info = components.get_info(component_id).unwrap();
            if SEND {
                assert!(
                    component_info.is_send_and_sync(),
                    "Send + Sync resource {} initialized as non_send. It may have been inserted non_send by accident.", component_info.name()
                );
            }

            let data = unsafe {
                BlobArray::with_capacity(
                    component_info.layout(),
                    component_info.drop(),
                    1
                )
            };
            
            ResourceData { 
                data, 
                is_present: false,
                added_ticks: UnsafeCell::new(Tick::new(0)),
                changed_ticks: UnsafeCell::new(Tick::new(0)),
                type_name: component_info.name(),
                #[cfg(feature = "std")] 
                origin_thread_id: None,
                changed_by: MaybeLocation::caller().map(UnsafeCell::new) }
        })
    }

    /// Gets read-only access to a resource, if it exists
    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<&ResourceData<SEND>> {
        self.resources.get(component_id)
    }

    /// Gets mutable access to a resource, if it exists
    #[inline]
    pub(crate) fn get_mut(&mut self, component_id: ComponentId) -> Option<&mut ResourceData<SEND>> {
        self.resources.get_mut(component_id)
    }
}
