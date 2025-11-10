#pragma once

#include "core/StringRef.hpp"

#include <type_traits>

/**
 * If there is no other specialization of DefaultHash for a given type, look for a hash function
 * on the type itself. Implementing a `hash()` method on a type is often significantly easier than
 * specializing DefaultHash
 */
template<typename T> struct DefaultHash {
  uint64_t operator()(const T &value) const {
    if constexpr (std::is_enum_v<T>) {
      return uint64_t(value);
    } else {
      return value.hash();
    }
  }
};

/**
 * Use the same hash function for const and non const variants of a type
 */
template<typename T> struct DefaultHash<const T> {
  uint64_t operator()(const T &value) const {
    return DefaultHash<T>{}(value);
  }
};

#define TRIVIAL_DEFAULT_INT_HASH(TYPE) \
  template<> struct DefaultHash<TYPE> { \
    uint64_t operator()(TYPE value) const { \
      return uint64_t(value); \
    } \
  }

TRIVIAL_DEFAULT_INT_HASH(int8_t);
TRIVIAL_DEFAULT_INT_HASH(uint8_t);
TRIVIAL_DEFAULT_INT_HASH(int16_t);
TRIVIAL_DEFAULT_INT_HASH(uint16_t);
TRIVIAL_DEFAULT_INT_HASH(int32_t);
TRIVIAL_DEFAULT_INT_HASH(uint32_t);
TRIVIAL_DEFAULT_INT_HASH(int64_t);
TRIVIAL_DEFAULT_INT_HASH(uint64_t);

template<> struct DefaultHash<float> {
  uint64_t operator()(float value) const {
    return uint64_t(*reinterpret_cast<uint32_t *>(&value));
  }
};

template<> struct DefaultHash<double> {
  uint64_t operator()(double value) const {
    return *reinterpret_cast<uint64_t *>(&value);
  }
};

template<> struct DefaultHash<bool> {
  uint64_t operator()(bool value) const {
    return uint64_t((value != false) * 1298191);
  }
};

inline uint64_t hash_string(StringRef str) {
  uint64_t hash = 5381;
  for (char c : str) {
    hash = hash * 33 + c;
  }
  return hash;
}

template<> struct DefaultHash<StringRef> {
  uint64_t operator()(StringRef value) const {
    return hash_string(value);
  }
};

template<typename T> struct DefaultHash<T *> {
  uint64_t operator()(const T *value) const {
    uintptr_t ptr = uintptr_t(value);
    uint64_t hash = uint64_t(ptr >> 4);
    return hash;
  }
};
