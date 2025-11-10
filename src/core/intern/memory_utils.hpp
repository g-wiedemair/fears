#pragma once

#include "core/sys_types.hpp"
#include "core/utildefines.hpp"

#include <cstdint>
#include <memory>

template<typename T> inline constexpr bool is_trivial_extended_v = std::is_trivial_v<T>;
template<typename T>
inline constexpr bool is_trivially_destructible_extended_v = is_trivial_extended_v<T> ||
                                                             std::is_trivially_destructible_v<T>;

/**
 * Inline buffers for small-object-optimization should be disabled by default for large objects
 */
constexpr int64_t default_inline_buffer_capacity(size_t element_size) {
  return (int64_t(element_size) < 100) ? 4 : 0;
}

template<typename T> void default_construct_n(T *ptr, int64_t n) {
  std::uninitialized_default_construct_n(ptr, n);
}

template<typename T> void destruct_n(T *ptr, int64_t n) {
  if (is_trivially_destructible_extended_v<T>) {
    return;
  }

  std::destroy_n(ptr, n);
}

//-------------------------------------------------------------------------------------------------

/**
 * This can be used to mark a constructor of an object that does not throw exceptions.
 * Other constructors can delegate to this constructor to make sure that the object lifetime starts
 * With this, the destructor of the object will be called, even when the remaining constructor
 * throws
 */
class NoExceptConstructor {};

//-------------------------------------------------------------------------------------------------

/**
 * An 'AlignedBuffer' is a byte array with at least the given size and alignment.
 * The buffer will not be initialized by the default constructor
 */
template<size_t Size, size_t Alignment> class AlignedBuffer {
  struct Empty {};
  struct alignas(Alignment) Sized {
    std::byte buffer_[Size > 0 ? Size : 1];
  };

  using BufferType = std::conditional_t<Size == 0, Empty, Sized>;
  FE_NO_UNIQUE_ADDRESS BufferType buffer_;

 public:
  operator void *() {
    return this;
  }
  operator const void *() const {
    return this;
  }

  void *ptr() {
    return this;
  }
  const void *ptr() const {
    return this;
  }
};

//-------------------------------------------------------------------------------------------------

/**
 * This can be used to reserve memory for C++ objects whose lifetime is different from the
 * lifetime of the object they are embedded in. It's used by containers with small buffer
 * optimization and hash table implementation
 */
template<typename T, int64_t Size = 1> class TypedBuffer {
 private:
  static constexpr size_t get_size() {
    if constexpr (Size == 0) {
      return 0;
    } else {
      return sizeof(T) * size_t(Size);
    }
  }

  static constexpr size_t get_alignment() {
    if constexpr (Size == 0) {
      return 1;
    } else {
      return alignof(T);
    }
  }

  FE_NO_UNIQUE_ADDRESS AlignedBuffer<get_size(), get_alignment()> buffer_;

 public:
  operator T *() {
    return static_cast<T *>(buffer_.ptr());
  }
  operator const T *() {
    return static_cast<const T *>(buffer_.ptr());
  }

  T &operator*() {
    return *static_cast<T *>(buffer_.ptr());
  }
  const T &operator*() const {
    return *static_cast<const T *>(buffer_.ptr());
  }

  T *ptr() {
    return static_cast<T *>(buffer_.ptr());
  }
  const T *ptr() const {
    return static_cast<const T *>(buffer_.ptr());
  }

  T &ref() {
    return *static_cast<T *>(buffer_.ptr());
  }
  const T &ref() const {
    return *static_cast<const T *>(buffer_.ptr());
  }
};

//-------------------------------------------------------------------------------------------------

/**
 * Helper variable that checks if Span<From> can be converted to Span<To> safely, whereby From and
 * To are pointers. Adding const and casting to a void pointer is allowed.
 * Casting up and down a class hierarchy generally is not allowed, because it might change the
 * pointer under some circumstances
 */
template<typename From, typename To>
inline constexpr bool is_span_convertible_pointer_v =
    // Make sure we are working with pointers
    std::is_pointer_v<From> && std::is_pointer_v<To> &&
    (  // No casting is necessary when both types are the same
        std::is_same_v<From, To> ||
        // Allow adding const to the underlying type
        std::is_same_v<std::remove_pointer_t<From>,
                       std::remove_const_t<std::remove_pointer_t<To>>> ||
        // Allow casting non-const pointers to void pointers
        (!std::is_const_v<std::remove_pointer_t<From>> && std::is_same_v<To, void *>) ||
        // Allow casting any pointer to const void pointer
        std::is_same_v<To, const void *>);
