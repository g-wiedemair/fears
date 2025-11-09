#pragma once
#include "core/assert.hpp"
#include "core/utildefines.hpp"

#include <cstdint>

/**
 * This struct provides an equality operator that returns true for all objects that compare equal
 * when one would use the '==' operator.
 */
template<typename T> struct DefaultEquality {
  template<typename T1, typename T2> bool operator()(const T1 &a, const T2 &b) const {
    return a == b;
  }
};

//-------------------------------------------------------------------------------------------------

/**
 * `0xffff...ffff` indicates an empty slot.
 * `0xffff...fffe` indicates a removed slot.
 *
 * Those specific values are used, because with them a single comparison is enough to check whether
 * a slot is occupied. The keys `0x0000...0000` and `0x0000...0001` also satisfy this constraint.
 * However, nullptr is much more likely to be used as valid key.
 */
template<typename Pointer> struct PointerKeyInfo {
  static Pointer get_empty() {
    return (Pointer)UINTPTR_MAX;
  }
};

//-------------------------------------------------------------------------------------------------

template<typename IntT> constexpr IntT ceil_division(const IntT x, const IntT y) {
  fassert(x >= 0);
  fassert(y >= 0);
  return x / y + ((x % y) != 0);
}

constexpr int64_t ceil_division_by_fraction(const int64_t x,
                                            const int64_t numerator,
                                            const int64_t denominator) {
  return int64_t(ceil_division(uint64_t(x) * uint64_t(denominator), uint64_t(numerator)));
}

constexpr int64_t total_slot_amount_for_usable_slots(const int64_t min_usable_slots,
                                                     const int64_t max_load_factor_numerator,
                                                     const int64_t max_load_factor_denominator) {
  return power_of_2_max(ceil_division_by_fraction(
      min_usable_slots, max_load_factor_numerator, max_load_factor_denominator));
}

/**
 * This is an abstraction for a fractional load factor
 * The hash table using this struct is assumed to use arrays with a size that is a power of two
 */
class LoadFactor {
 private:
  uint8_t numerator_;
  uint8_t denominator_;

 public:
  constexpr LoadFactor(uint8_t numerator, uint8_t denominator)
      : numerator_(numerator), denominator_(denominator) {
    fassert(numerator > 0);
    fassert(numerator < denominator);
  }

  constexpr void compute_total_and_usable_slots(int64_t min_total_slots,
                                                int64_t min_usable_slots,
                                                int64_t *r_total_slots,
                                                int64_t *r_usable_slots) const {
    todo();
  }

  static constexpr int64_t compute_total_slots(int64_t min_usable_slots,
                                               uint8_t numerator,
                                               uint8_t denominator) {
    return total_slot_amount_for_usable_slots(min_usable_slots, numerator, denominator);
  }
};
