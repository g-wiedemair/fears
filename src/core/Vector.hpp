#pragma once

#include "core/Span.hpp"
#include "core/allocator.hpp"
#include "core/intern/memory_utils.hpp"

#include <vector>

/**
 * A Vector<T> is a dynamically growing contiguous array for values of type T. It is designed to be
 * a more convenient and efficient replacement for `std::vector`.
 *
 * A Vector supports efficient insertion and removal at the end (O(1) amortized). Removal in other
 * places takes O(n) time, because all elements afterwardshave to be moved. If the order of
 * elements is not important, `remove_and_reorder` can be used instead of `remove` for better
 * performance.
 *
 * The improved efficiency is mainly achieved by supporting small buffer optimization. As long as
 * the number of elements in the Vector does not become larger than InlineBufferCapacity, no
 * allocation is done.
 */
template<
    /**
     * Type of the values stored in this vector. It has to be movable
     */
    typename T,
    /**
     * The numbeer of values that can be stored in this vector, without doing a heap allocation.
     * Sometimes it makes sense to increase this value a lot. The memory in the inline buffer is
     * not initialized when it is not needed.
     */
    int64_t InlineBufferCapacity = default_inline_buffer_capacity(sizeof(T)),
    /**
     * The allocator used by this vector
     */
    typename Allocator = GuardedAllocator>
class Vector {
 public:
  using value_type = T;
  using pointer = T *;
  using const_pointer = const T *;
  using reference = T &;
  using const_reference = const T &;
  using iterator = T *;
  using const_iterator = const T *;
  using size_type = int64_t;
  using allocator_type = Allocator;

 private:
  /**
   * Use pointers instead of storing the size explicitly. This reduces the number of instructions
   * in `append`. The pointers might point to the memory in the inline buffer
   */
  T *begin_;
  T *end_;
  T *capacity_end_;

  FE_NO_UNIQUE_ADDRESS Allocator allocator_;

  FE_NO_UNIQUE_ADDRESS TypedBuffer<T, InlineBufferCapacity> inline_buffer_;

  /** Store the size of vector explicitly in debug builds */
#ifndef NDEBUG
  int64_t debug_size_;
#  define UPDATE_VECTOR_SIZE(ptr) (ptr)->debug_size_ = int64_t((ptr)->end_ - (ptr)->begin_)
#else
#  define UPDATE_VECTOR_SIZE(ptr) ((void)0)
#endif

  template<typename OtherT, int64_t OtherInlineBufferCapacity, typename OtherAllocator>
  friend class Vector;

  /** Required in case `T` is an incomplete type */
  static constexpr bool is_nothrow_move_constructible() {
    if constexpr (InlineBufferCapacity == 0) {
      return true;
    } else {
      return std::is_nothrow_move_constructible_v<T>;
    }
  }

 public:
  Vector(Allocator allocator = {}) noexcept : allocator_(allocator) {
    begin_ = inline_buffer_;
    end_ = begin_;
    capacity_end_ = begin_ + InlineBufferCapacity;
    UPDATE_VECTOR_SIZE(this);
  }

  Vector(NoExceptConstructor, Allocator allocator = {}) noexcept : Vector(allocator) {}

  /// Create a Vector with a specific size
  /// The elements will be default constructed.
  explicit Vector(int64_t size, Allocator allocator = {})
      : Vector(NoExceptConstructor(), allocator) {
    this->resize(size);
  }

  /// Create a Vector filled with a specific value
  Vector(int64_t size, const T &value, Allocator allocator = {})
      : Vector(NoExceptConstructor(), allocator) {
    this->resize(size, value);
  }

  /// Create a Vector from a span. The values in the Vector are copy constructed
  template<typename U, ENABLE_IF((std::is_convertible_v<U, T>))>
  Vector(Span<U> values, Allocator allocator = {}) : Vector(NoExceptConstructor(), allocator) {
    todo();
  }

  // template<typename U, ENABLE_IF((std::is_convertible_v<U, T>))>
  // explicit Vector(MutableSpan<U> values, Allocator allocator = {})
  //     : Vector(values.as_span(), allocator) {}

  template<typename U, ENABLE_IF((std::is_convertible_v<U, T>))>
  Vector(const std::initializer_list<U> &values) : Vector(Span<U>(values)) {}

  Vector(const std::initializer_list<T> &values) : Vector(Span<T>(values)) {}

  template<typename U, size_t N, ENABLE_IF((std::is_convertible_v<U, T>))>
  Vector(const std::array<U, N> &values) : Vector(Span(values)) {}

  Vector(const Vector &other) : Vector(other.as_span(), other.allocator_) {}

  template<int64_t OtherInlineBufferCapacity>
  Vector(const Vector<T, OtherInlineBufferCapacity, Allocator> &other)
      : Vector(other.as_span(), other.allocator_) {}

  template<int64_t OtherInlineBufferCapacity>
  Vector(Vector<T, OtherInlineBufferCapacity, Allocator> &&other) noexcept(
      is_nothrow_move_constructible())
      : Vector(NoExceptConstructor(), other.allocator_) {
    todo();
  }

  // Vector(const VectorData<T, Allocator> &data) : Vector(data.allocator) {
  //   todo();
  // }

  ~Vector() {
    destruct_n(begin_, this->size());
    if (!this->is_inline()) {
      allocator_.deallocate(begin_);
    }
  }

  Vector &operator=(const Vector &other) {
    todo();
    return *this;
  }

  Vector &operator=(Vector &&other) {
    todo();
    return *this;
  }

  const T &operator[](int64_t index) const {
    fassert(index >= 0);
    fassert(index < this->size());
    return begin_[index];
  }

  T &operator[](int64_t index) {
    fassert(index >= 0);
    fassert(index < this->size());
    return begin_[index];
  }

  operator Span<T>() const {
    return Span<T>(begin_, this->size());
  }

  // operator MutableSpan<T>() {
  //   return MutableSpan<T>(begin_, this->size());
  // }

  Span<T> as_span() const {
    return *this;
  }

  void reserve(const int64_t min_capacity) {
    if (min_capacity > this->capacity()) {
      // this->realloc_to_at_least(min_capacity);
      todo();
    }
  }

  void resize(const int64_t new_size) {
    todo();
  }

  void resize(const int64_t new_size, const T &value) {
    todo();
  }

  void reinitialize(const int64_t new_size) {
    todo();
  }

  void clear() {
    todo();
  }

  void clear_and_shring() {
    todo();
  }

  void append(const T &value) {
    this->append_as(value);
  }
  void append(const T &&value) {
    this->append_as(std::move(value));
  }
  template<typename... ForwardValue> void append_as(ForwardValue &&...value) {
    this->ensure_space_for_one();
    this->append_unchecked_as(std::forward<ForwardValue>(value)...);
  }

  int64_t append_and_get_index(const T &value) {
    return this->append_and_get_index_as(value);
  }
  int64_t append_and_get_index(T &&value) {
    return this->append_and_get_index_as(std::move(value));
  }
  template<typename... ForwardValue> int64_t append_and_get_index_as(ForwardValue &&...value) {
    const int64_t index = this->size();
    this->append_as(std::forward<ForwardValue>(value)...);
    return index;
  }

  void append_non_duplicates(const T &value) {
    todo();
  }

  void append_unchecked(const T &value) {
    todo();
  }
  void append_unchecked(T &&value) {
    this->append_unchecked_as(std::move(value));
  }
  template<typename... ForwardValue> void append_unchecked_as(ForwardValue &&...value) {
    fassert(end_ < capacity_end_);
    new (end_) T(std::forward<ForwardValue>(value)...);
    end_++;
    UPDATE_VECTOR_SIZE(this);
  }

  /**
   * Insert values at the beginning of the vector
   * This has to move all the other elements, so it is not very efficient.
   */
  void prepend(const T &value) {
    todo();
  }
  void prepend(T &&value) {
    todo();
  }
  void prepend(Span<T> values) {
    todo();
  }
  template<typename InputIt> void prepend(InputIt first, InputIt last) {
    todo();
  }

  T *begin() {
    return begin_;
  }
  T *end() {
    return end_;
  }

  int64_t size() const {
    const int64_t current_size = int64_t(end_ - begin_);
    fassert(debug_size_ == current_size);
    return current_size;
  }

  bool is_empty() const {
    return begin_ == end_;
  }

  int64_t capacity() const {
    return int64_t(capacity_end_ - begin_);
  }

  friend bool operator==(const Vector &a, const Vector &b) {
    return a.as_span() == b.as_span();
  }
  friend bool operator!=(const Vector &a, const Vector &b) {
    return !(a == b);
  }

  bool is_inline() const {
    return begin_ == inline_buffer_.ptr();
  }

 private:
  void ensure_space_for_one() {
    if (UNLIKELY(end_ >= capacity_end_)) {
      this->realloc_to_at_least(this->size() + 1);
    }
  }

  void realloc_to_at_least(const int64_t min_capacity) {
    if (this->capacity() >= min_capacity) {
      return;
    }

    todo();
  }
};
