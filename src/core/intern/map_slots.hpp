#pragma once
#include "core/assert.hpp"
#include "core/intern/hash_tables.hpp"
#include "core/intern/memory_utils.hpp"
#include "core/sys_types.hpp"

#include <type_traits>

/**
 * The simplest possible map slot.
 * It stores the slot state and the optional key and value instances in separate variables
 */
template<typename Key, typename Value> class SimpleMapSlot {
 private:
  enum State : u8 {
    Empty = 0,
    Occupied = 1,
    Removed = 2,
  };

  State state_;
  TypedBuffer<Key> key_buffer_;
  TypedBuffer<Value> value_buffer_;

 public:
  SimpleMapSlot() {
    state_ = Empty;
  }

  ~SimpleMapSlot() {
    if (state_ == Occupied) {
      key_buffer_.ref().~Key();
      value_buffer_.ref().~Value();
    }
  }

  SimpleMapSlot(const SimpleMapSlot &other) {
    todo();
  }

  SimpleMapSlot(SimpleMapSlot &&other) noexcept(std::is_nothrow_move_constructible_v<Key> &&
                                                std::is_nothrow_move_constructible_v<Value>) {
    todo();
  }

  Key *key() {
    return key_buffer_;
  }
  const Key *key() const {
    return key_buffer_;
  }

  Value *value() {
    return value_buffer_;
  }
  const Value *value() const {
    return value_buffer_;
  }

  bool is_empty() const {
    return state_ == Empty;
  }

  bool is_occupied() const {
    return state_ == Occupied;
  }

  template<typename Hash> uint64_t get_hash(const Hash &hash) {
    fassert(this->is_occupied());
    return hash(*key_buffer_);
  }

  template<typename ForwardKey, typename IsEqual>
  bool contains(const ForwardKey &key, const IsEqual &is_equal, uint64_t /*hash*/) const {
    if (state_ == Occupied) {
      return is_equal(key, *key_buffer_);
    }
    return false;
  }

  template<typename ForwardKey, typename... ForwardValue>
  void occupy(ForwardKey &&key, uint64_t hash, ForwardValue &&...value) {
    fassert(!this->is_occupied());
    new (&value_buffer_) Value(std::forward<ForwardValue>(value)...);
    this->occupy_no_value(std::forward<ForwardKey>(key), hash);
    state_ = Occupied;
  }

  template<typename ForwardKey> void occupy_no_value(ForwardKey &&key, uint64_t /*hash*/) {
    fassert(!this->is_occupied());
    try {
      new (&key_buffer_) Key(std::forward<ForwardKey>(key));
    }
    catch (...) {
      value_buffer_.ref().~Value();
      throw;
    }
    state_ = Occupied;
  }
};

/**
 * An IntrusiveMapSlot uses two special values of the key to indicate whether the slot is empty
 * or removed. This saves some memory in all cases and is more efficient in many cases. The
 * KeyInfo type indicates which specific values are used. An example for a KeyInfo
 * implementation is PointerKeyInfo.
 *
 * The special key values are expected to be trivially destructible.
 */
template<typename Key, typename Value, typename KeyInfo> class IntrusiveMapSlot {
 private:
  Key key_ = KeyInfo::get_empty();
  TypedBuffer<Value> value_buffer_;

 public:
  IntrusiveMapSlot() = default;

  ~IntrusiveMapSlot() {
    if (KeyInfo::is_not_empty_or_removed(key_)) {
      value_buffer_.ref().~Value();
    }
  }

  IntrusiveMapSlot(const IntrusiveMapSlot &other) : key_(other.key_) {
    todo();
  }

  IntrusiveMapSlot(IntrusiveMapSlot &&other) noexcept : key_(other.key_) {
    todo();
  }
};

/**
 * Use SimpleMapSlot by default, because it is the smallest slot type
 */
template<typename Key, typename Value> struct DefaultMapSlot {
  using type = SimpleMapSlot<Key, Value>;
};

/**
 * Use a special slot type for pointer keys, because we can store whether a slot is empty or
 * removed with special pointer values.
 */
template<typename Key, typename Value> struct DefaultMapSlot<Key *, Value> {
  using type = IntrusiveMapSlot<Key *, Value, PointerKeyInfo<Key *>>;
};
