#pragma once
#include "core/allocator.hpp"
#include "core/assert.hpp"
#include "core/intern/memory_utils.hpp"

#include <cstdint>

/**
 * A Array<T> is a container for a fixed size array the size of which is NOT known at compile time
 * If the size is known at compile time, std::array<T, N> should be used instead.
 *
 * Array should usually be used instead of Vector whenever the number of elements is known at
 * construction time. NOte however, that Array will default construct all elements when initialized
 * with the size-constructor. For trivial types, this does nothing. In all other cases, this adds
 * overhead.
 *
 * A main benefit of using Array over Vector is that it expresses the intent of the developer
 * better. It indicates that the size of the data structure is not expected to change. Furthermore
 * you can be more certain that an array does not over-allocate.
 *
 * Array supports small object optimization to improve performance when the size turns out to be
 * small at run-time.
 */
template<
    /**
     * The type of the values stored in the array
     */
    typename T,
    /**
     * The number of values that can be stored in the array, without doing a heap allocation
     */
    int64_t InlineBufferCapacity = default_inline_buffer_capacity(sizeof(T)),
    /**
     * The allocator used by this array
     */
    typename Allocator = GuardedAllocator>
class Array {
 public:
  using value_type = T;
  using pointer = T *;
  using const_pointer = const T *;
  using reference = T &;
  using const_reference = const T &;
  using iterator = T *;
  using const_iterator = const T *;
  using size_type = int64_t;

 private:
  /** The beginning of the array. It might point into the inline buffer */
  T *data_;

  /** Number of elements in the array >*/
  int64_t size_;

  /** Used for allocations when the inline buffer is too small */
  FE_NO_UNIQUE_ADDRESS Allocator allocator_;

  /** A placeholder buffer that will remain uninitialized until it is used */
  FE_NO_UNIQUE_ADDRESS TypedBuffer<T, InlineBufferCapacity> inline_buffer_;

 public:
  Array(Allocator allocator = {}) noexcept : allocator_(allocator) {
    data_ = inline_buffer_;
    size_ = 0;
  }

  Array(NoExceptConstructor, Allocator allocator = {}) noexcept : Array(allocator) {}

  /// Create a new array with the given size. All values will be default constructed
  explicit Array(int64_t size, Allocator allocator = {})
      : Array(NoExceptConstructor(), allocator) {
    data_ = this->get_buffer_for_size(size);
    default_construct_n(data_, size);
    size_ = size;
  }

  Array(const Array &other) {
    todo();
  }

  Array(Array &&other) noexcept(std::is_nothrow_move_constructible_v<T>)
      : Array(NoExceptConstructor(), other.allocator_) {
    todo();
  }

  ~Array() {
    destruct_n(data_, size_);
    this->deallocate_if_not_inline(data_);
  }

  Array &operator=(Array &&other) noexcept(std::is_nothrow_move_constructible_v<T>) {
    todo();
  }

  T &operator[](int64_t index) {
    fassert(index >= 0);
    fassert(index < size_);
    return data_[index];
  }
  const T &operator[](int64_t index) const {
    fassert(index >= 0);
    fassert(index < size_);
    return data_[index];
  }

  T *begin() {
    return data_;
  }
  const T *begin() const {
    return data_;
  }

  T *end() {
    return data_ + size_;
  }
  const T *end() const {
    return data_ + size_;
  }

  static int64_t inline_buffer_capacity() {
    return InlineBufferCapacity;
  }

  Allocator &allocator() {
    return allocator_;
  }
  const Allocator &allocator() const {
    return allocator_;
  }

 private:
  T *get_buffer_for_size(int64_t size) {
    if (size <= InlineBufferCapacity) {
      return inline_buffer_;
    }
    return this->allocate(size);
  }

  T *allocate(int64_t size) {
    return static_cast<T *>(allocator_.allocate(size_t(size) * sizeof(T), alignof(T), AT));
  }

  void deallocate_if_not_inline(T *ptr) {
    if (ptr != inline_buffer_) {
      allocator_.deallocate(ptr);
    }
  }
};
