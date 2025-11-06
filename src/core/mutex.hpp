#pragma once

#include <mutex>

#ifdef WITH_TBB

/**
 * Mutex should be used as the default mutex in feap. It implements a subset of the API
 * of std::mutex but has overall better guaranteed properties. It can be used with RAII helpers
 * like std::lock_guard. However, it is not compatible with e.g. std::condition_variable. So one
 * still has to use std::mutex for that case.
 *
 * The mutex provided by TBB has these properties:
 * - It's as fast as a spin-lock in the non-contended case, i.e. when no other thread is trying to
 *   lock the mutex at the same time.
 * - In the contended case, it spins a couple of times but then blocks to avoid draining system
 *   resources by spinning for a long time.
 * - It's only 1 byte large, compared to e.g. 40 bytes when using the std::mutex of GCC. This makes
 *   it more feasible to have many smaller mutexes which can improve scalability of algorithms
 *   compared to using fewer larger mutexes. Also it just reduces "memory slop" across feap.
 * - It is *not* a fair mutex, i.e. it's not guaranteed that a thread will ever be able to lock the
 *   mutex when there are always more than one threads that try to lock it. In the majority of
 *   cases, using a fair mutex just causes extra overhead without any benefit. std::mutex is not
 *   guaranteed to be fair either.
 */
using Mutex = tbb::mutex;

/* If this is not true anymore at some point, the comment above needs to be updated. */
static_assert(sizeof(Mutex) == 1);

#else

using Mutex = std::mutex;

#endif
