#pragma once

#include "core/HashMap.hpp"
#include "core/Vector.hpp"

/**
 * This class implements a map that also provides access to all stored values
 * in a deterministic order. The values are kept in a Vector and the mapping is done
 * with a HashMap from keys to indexes in that vector
 */
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
    typename Slot = typename DefaultMapSlot<Key, size_t>::type,
    /**
     * The allocator used by this map
     */
    typename Allocator = GuardedAllocator,
    /**
     * The Type of map
     */
    typename MapType = HashMap<Key,
                               size_t,
                               InlineBufferCapacity,
                               ProbingStrategy,
                               Hash,
                               IsEqual,
                               Slot,
                               Allocator>,
    /**
     * The Type of vector
     */
    typename VectorType = Vector<Value, InlineBufferCapacity, Allocator>>
class VectorMap {
 private:
  MapType map_;
  VectorType vector_;

 public:
  using key_type = Key;
  using value_type = typename VectorType::value_type;
  using size_type = typename VectorType::size_type;

  using iterator = typename VectorType::iterator;
  using const_iterator = typename VectorType::const_iterator;

  VectorMap(Allocator allocator = {}) noexcept : map_(allocator), vector_(allocator) {}

  VectorMap(NoExceptConstructor, Allocator allocator = {}) noexcept : VectorMap(allocator) {}

  ~VectorMap() {
    map_.~MapType();
    vector_.~VectorType();
  }

  VectorMap(const VectorMap &other) : map_(other.map_), vector_(other.vector_) {}

  VectorMap(VectorMap &&other) noexcept {
    todo();
  }

  bool add(const Key &key, const Value &value) {
    int64_t index = vector_.append_and_get_index(value);
    return map_.add(key, index);
  }

  iterator begin() {
    return vector_.begin();
  }
  iterator end() {
    return vector_.end();
  }

  size_type size() const {
    return vector_.size();
  }

  bool empty() const {
    return vector_.empty();
  }

  void reserve(size_type num_entries) {
    map_.reserve(num_entries);
    vector_.reserve(num_entries);
  }
};
