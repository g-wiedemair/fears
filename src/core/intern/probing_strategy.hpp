#pragma once

#include <cstdint>

/**
 * This is the probing strategy used by CPython
 *
 * It is very fast when the original hash value is good.
 * If there are collissions, more bits of the hash value are taken into account
 *
 * LinearSteps: Can be set to something larger than 1 for improved cache performance in some cases
 * PreShuffle: When true, the initial call to next() will be done to the constructor
 *             This can help when the hash function has put little informaiton into the lower bits
 */
template<uint_fast64_t LinearSteps = 1, bool PreShuffle = false> class PythonProbingStrategy {
 private:
  uint64_t hash_;
  uint64_t perturb_;

 public:
  PythonProbingStrategy(const uint64_t hash) : hash_(hash), perturb_(hash) {
    if (PreShuffle) {
      this->next();
    }
  }

  void next() {
    perturb_ >>= 5;
    hash_ = 5 * hash_ + 1 + perturb_;
  }

  uint64_t get() const {
    return hash_;
  }

  int64_t linear_steps() const {
    return LinearSteps;
  }
};

using DefaultProbingStrategy = PythonProbingStrategy<>;

/**
 * Both macros together form a loop that iterates over slot indices in a hash table with a
 * power-of-two size
 */

// clang-format off
#define SLOT_PROBING_BEGIN(PROBING_STRATEGY, HASH, MASK, R_SLOT_INDEX) \
  PROBING_STRATEGY probing_strategy(HASH); \
  do { \
    i64 linear_offset = 0; \
    u64 current_hash = probing_strategy.get(); \
  do { \
    i64 R_SLOT_INDEX = i64((current_hash + u64(linear_offset)) & MASK);

#define SLOT_PROBING_END() \
    } while (++linear_offset < probing_strategy.linear_steps()); \
    probing_strategy.next(); \
  } while (true)
// clang-format on
