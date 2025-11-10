#pragma once

#include "core/IndexRange.hpp"
#include "core/assert.hpp"
#include "core/intern/memory_utils.hpp"
#include "core/utildefines.hpp"

#include <cstdint>
#include <vector>

/**
 * References an array of type T that is owned by someone else.
 * The data in the array cannot be modified.
 */
template<typename T> class Span {
 public:
  using value_type = T;
  using pointer = T *;
  using const_pointer = const T *;
  using reference = T &;
  using const_reference = const T &;
  using iterator = const T *;
  using size_type = int64_t;

 private:
  const T *data_ = nullptr;
  int64_t size_ = 0;

 public:
  constexpr Span() = default;

  constexpr Span(const T *start, int64_t size) : data_(start), size_(size) {
    fassert(size >= 0);
  }

  template<typename U, ENABLE_IF((is_span_convertible_pointer_v<U, T>))>
  constexpr Span(const U *start, int64_t size)
      : data_(static_cast<const T *>(start)), size_(size) {
    fassert(size >= 0);
  }

  /**
   * Reference an initializer list. Note that the data in the initializer_list is only valid until
   * the expression containing it is fully computed
   */
  constexpr Span(const std::initializer_list<T> &list)
      : Span(list.begin(), int64_t(list.size())) {}

  constexpr Span(const std::vector<T> &vector) : Span(vector.data(), int64_t(vector.size())) {}

  template<std::size_t N> constexpr Span(const std::array<T, N> &array) : Span(array.data(), N) {}

  template<typename U, ENABLE_IF((is_span_convertible_pointer_v<U, T>))>
  constexpr Span(Span<U> span) : data_(static_cast<const T *>(span.data())), size_(span.size()) {}

  /**
   * Returns a contiguous part of the array.
   * This invokes undefined behavior when the start or size is negative
   */
  constexpr Span slice(int64_t start, int64_t size) const {
    fassert(start >= 0);
    fassert(size >= 0);
    fassert(start + size <= size_ || size == 0);
    return Span(data_ + start, size);
  }

  constexpr Span slice(IndexRange range) const {
    return this->slice(range.start(), range.size());
  }

  friend bool operator==(const Span<T> a, const Span<T> b) {
    todo();
  }
  friend bool operator!=(const Span<T> a, const Span<T> b) {
    return !(a == b);
  }
};
