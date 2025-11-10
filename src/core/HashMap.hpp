#pragma once

#include "core/allocator.hpp"
#include "core/array.hpp"
#include "core/intern/hash.hpp"
#include "core/intern/hash_tables.hpp"
#include "core/intern/map_slots.hpp"
#include "core/intern/memory_utils.hpp"
#include "core/intern/probing_strategy.hpp"

#include <cstdint>

/**
 * A key-value-pair stored in a Map. This is used when looping over Map.items()
 */
template<typename Key, typename Value> struct MapItem {
  const Key &key;
  const Value &value;
};

/**
 * Same as MapItem, but the value is mutable
 */
template<typename Key, typename Value> struct MutableMapItem {
  const Key &key;
  Value &value;

  operator MapItem<Key, Value>() const {
    return {this->key, this->value};
  }
};

template<
    /**
     * Type of the keys stored in the map. Keys have to be movable.
     * Furthermore, the hash and is-equal functions have to support it.
     */
    typename Key,
    /**
     * Type of the value that is stored per key. It has to be movable as well
     */
    typename Value,
    /**
     * The minimum number of elements that can be stored in this Map without doing a heap
     * allocation
     */
    int64_t InlineBufferCapacity = default_inline_buffer_capacity(sizeof(Key) + sizeof(Value)),
    /**
     * The strategy used to deal with collisions
     */
    typename ProbingStrategy = DefaultProbingStrategy,
    /**
     * The hash function used to hash the keys
     */
    typename Hash = DefaultHash<Key>,
    /**
     * The equality operator used to compare keys
     */
    typename IsEqual = DefaultEquality<Key>,
    /**
     * This is what will actually be stored in the hash table array
     * At a minimum a slot has to be able to hold a key, a value and information about whether the
     * slot is empty, occupied or removed
     */
    typename Slot = typename DefaultMapSlot<Key, Value>::type,
    /**
     * The allocator used by this map
     */
    typename Allocator = GuardedAllocator>
class HashMap {
 public:
  using size_type = int64_t;
  using Item = MapItem<Key, Value>;
  using MutableItem = MutableMapItem<Key, Value>;

 private:
  /**
   * Slots are either empty, occupied or removed.
   * The number of occupied slots can be computed.
   */
  int64_t removed_slots_;
  int64_t occupied_and_removed_slots_;

  /**
   * The maximum number of slots that can be used until the set has to grow.
   * This is the total number of slots times the max load factor.
   */
  int64_t usable_slots_;

  /**
   * The number of slots minus one.
   * This is a bit mask that can be used to turn any integer in a valid slot index
   */
  uint64_t slot_mask_;

  FE_NO_UNIQUE_ADDRESS Hash hash_;

  FE_NO_UNIQUE_ADDRESS IsEqual is_equal_;

  // The max load factor is 1/2 = 50% by default
#define LOAD_FACTOR 1, 2
  static constexpr LoadFactor max_load_factor_ = LoadFactor(LOAD_FACTOR);
  using SlotArray =
      Array<Slot, LoadFactor::compute_total_slots(InlineBufferCapacity, LOAD_FACTOR), Allocator>;
#undef LOAD_FACTOR

  // This is the array that contains the actual slots.
  SlotArray slots_;

  // Iterate over a slot index sequence for a given hash
#define MAP_SLOT_PROBING_BEGIN(HASH, R_SLOT) \
  SLOT_PROBING_BEGIN (ProbingStrategy, HASH, slot_mask_, SLOT_INDEX) \
    auto &R_SLOT = slots_[SLOT_INDEX];
#define MAP_SLOT_PROBING_END() SLOT_PROBING_END()

 public:
  HashMap(Allocator allocator = {}) noexcept
      : removed_slots_(0),
        occupied_and_removed_slots_(0),
        usable_slots_(0),
        slot_mask_(0),
        hash_(),
        is_equal_(),
        slots_(1, allocator) {}

  HashMap(NoExceptConstructor, Allocator allocator = {}) noexcept : HashMap(allocator) {}

  ~HashMap() = default;

  HashMap(const HashMap &) = default;

  HashMap(HashMap &&other) noexcept(std::is_nothrow_move_constructible_v<SlotArray>) {
    todo();
  }

  HashMap &operator=(const HashMap &other) {
    todo();
    return *this;
  }

  HashMap &operator=(HashMap &&other) {
    todo();
    return *this;
  }

  bool add(const Key &key, const Value &value) {
    return this->add_as(key, value);
  }
  bool add(const Key &key, Value &&value) {
    return this->add_as(key, value);
  }
  bool add(Key &&key, const Value &value) {
    return this->add_as(key, value);
  }
  bool add(Key &&key, Value &&value) {
    return this->add_as(key, value);
  }
  template<typename ForwardKey, typename... ForwardValue>
  bool add_as(ForwardKey &&key, ForwardValue &&...value) {
    return this->add_impl(
        std::forward<ForwardKey>(key), hash_(key), std::forward<ForwardValue>(value)...);
  }

  /* Common base class for all iterators below */
  struct BaseIterator {
   public:
    using iterator_category = std::forward_iterator_tag;
    using difference_type = std::ptrdiff_t;

   protected:
    Slot *slots_;
    int64_t total_slots_;
    int64_t current_slot_;

    friend class HashMap;

   public:
    BaseIterator(const Slot *slots, const int64_t total_slots, const int64_t current_slot)
        : slots_(const_cast<Slot *>(slots)),
          total_slots_(total_slots),
          current_slot_(current_slot) {}

    BaseIterator &operator++() {
      while (++current_slot_ < total_slots_) {
        if (slots_[current_slot_].is_occupied()) {
          break;
        }
      }
      return *this;
    }
    BaseIterator operator++(int) {
      BaseIterator copied_iterator = *this;
      ++(*this);
      return copied_iterator;
    }

    friend bool operator!=(const BaseIterator &a, const BaseIterator &b) {
      fassert(a.slots_ == b.slots_);
      fassert(a.total_slots_ == b.total_slots_);
      return a.current_slot_ != b.current_slot_;
    }
    friend bool operator==(const BaseIterator &a, const BaseIterator &b) {
      return !(a != b);
    }

   protected:
    Slot &current_slot() const {
      return slots_[current_slot_];
    }
  };

  /**
   * A utility iterator that reduces the amount of code when implementing the actual iterators
   */
  template<typename SubIterator> class BaseIteratorRange : public BaseIterator {
   public:
    BaseIteratorRange(const Slot *slots, int64_t total_slots, int64_t current_slot)
        : BaseIterator(slots, total_slots, current_slot) {}

    SubIterator begin() const {
      for (int64_t i = 0; i < this->total_slots_; i++) {
        if (this->slots_[i].is_occupied()) {
          return SubIterator(this->slots_, this->total_slots_, i);
        }
      }
      return this->end();
    }
    SubIterator end() const {
      return SubIterator(this->slots_, this->total_slots_, this->total_slots_);
    }
  };

  class ValueIterator final : public BaseIteratorRange<ValueIterator> {
   public:
    using value_type = Value;
    using pointer = const Value *;
    using reference = const Value &;

    ValueIterator(const Slot *slots, int64_t total_slots, int64_t current_slot)
        : BaseIteratorRange<ValueIterator>(slots, total_slots, current_slot) {}

    const Value &operator*() const {
      return *this->current_slot().value();
    }
  };

  /// Returns an iterator over all values in the map.
  /// The iterator is invalidated, when the map is changed
  ValueIterator values() const & {
    return ValueIterator(slots_.data(), slots_.size(), 0);
  }

  int64_t size() const {
    return occupied_and_removed_slots_ - removed_slots_;
  }

 private:
  void ensure_can_add() {
    if (occupied_and_removed_slots_ >= usable_slots_) {
      this->realloc_and_reinsert(this->size() + 1);
      fassert(occupied_and_removed_slots_ < usable_slots_);
    }
  }

  void realloc_and_reinsert(int64_t min_usable_slots) {
    int64_t total_slots, usable_slots;
    max_load_factor_.compute_total_and_usable_slots(
        SlotArray::inline_buffer_capacity(), min_usable_slots, &total_slots, &usable_slots);
    fassert(total_slots >= 1);
    const uint64_t new_slot_mask = uint64_t(total_slots) - 1;

    // Optimize the case when the map was empty beforehand
    if (this->size() == 0) {
      try {
        slots_.reinitialize(total_slots);
      }
      catch (...) {
        this->noexcept_reset();
        throw;
      }
      removed_slots_ = 0;
      occupied_and_removed_slots_ = 0;
      usable_slots_ = usable_slots;
      slot_mask_ = new_slot_mask;
      return;
    }

    SlotArray new_slots(total_slots);

    try {
      for (Slot &slot : slots_) {
        if (slot.is_occupied()) {
          todo();
        }
      }
      slots_ = std::move(new_slots);
    }
    catch (...) {
      todo();
      throw;
    }

    occupied_and_removed_slots_ -= removed_slots_;
    usable_slots_ = usable_slots;
    removed_slots_ = 0;
    slot_mask_ = new_slot_mask;
  }

  void noexcept_reset() noexcept {
    todo();
  }

  template<typename ForwardKey, typename... ForwardValue>
  bool add_impl(ForwardKey &&key, uint64_t hash, ForwardValue &&...value) {
    this->ensure_can_add();

    MAP_SLOT_PROBING_BEGIN (hash, slot) {
      if (slot.is_empty()) {
        slot.occupy(std::forward<ForwardKey>(key), hash, std::forward<ForwardValue>(value)...);
        fassert(hash_(*slot.key()) == hash);
        occupied_and_removed_slots_++;
        return true;
      }
      if (slot.contains(key, is_equal_, hash)) {
        return false;
      }
    }
    MAP_SLOT_PROBING_END();
  }
};
