#pragma once

#include "core/assert.hpp"
#include "core/intern/RandomAccessIterator.hpp"

#include <cstdint>

template<typename T> class Span;

class IndexRange {
 private:
  int64_t start_ = 0;
  int64_t size_ = 0;

 public:
  constexpr IndexRange() = default;

  constexpr explicit IndexRange(int64_t size) : size_(size) {
    fassert(size >= 0);
  }

  constexpr IndexRange(const int64_t start, const int64_t size) : start_(start), size_(size) {
    fassert(start >= 0);
    fassert(size >= 0);
  }

  constexpr static IndexRange from_begin_size(const int64_t begin, const int64_t size) {
    return IndexRange(begin, size);
  }

  constexpr static IndexRange from_begin_end(const int64_t begin, const int64_t end) {
    return IndexRange(begin, end - begin);
  }

  class Iterator : public RandomAccessIterator<Iterator> {
   public:
    using value_type = int64_t;
    using pointer = const int64_t *;
    using reference = int64_t;

   private:
    int64_t current_;

   public:
    constexpr explicit Iterator(int64_t current) : current_(current) {}

    constexpr int64_t operator*() const {
      return current_;
    }

    const int64_t &iter_prop() const {
      return current_;
    }
  };

  constexpr Iterator begin() const {
    return Iterator(start_);
  }

  constexpr Iterator end() const {
    return Iterator(start_ + size_);
  }

  constexpr int64_t operator[](const int64_t index) const {
    fassert(index >= 0);
    fassert(index < this->size());
    return start_ + index;
  }

  constexpr friend bool operator==(IndexRange a, IndexRange b) {
    return (a.size_ == b.size_) && (a.start_ == b.start_ || a.size_ == 0);
  }
  constexpr friend bool operator!=(IndexRange a, IndexRange b) {
    return !(a == b);
  }

  constexpr int64_t size() const {
    return size_;
  }

  constexpr bool is_empty() const {
    return size_ == 0;
  }

  constexpr int64_t start() const {
    return start_;
  }
};
