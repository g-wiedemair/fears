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

#define LOAD_FACTOR 1, 2
  static constexpr LoadFactor max_load_factor_ = LoadFactor(LOAD_FACTOR);
  using SlotArray =
      Array<Slot, LoadFactor::compute_total_slots(InlineBufferCapacity, LOAD_FACTOR), Allocator>;
#undef LOAD_FACTOR

  SlotArray slots_;

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
};
