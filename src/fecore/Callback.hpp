#pragma once

#include "core/Vector.hpp"
#include "fecore/fecore_api.hpp"

#include <cstdint>

class FeModel;

/* clang-format off */
enum CallbackEvents {
  CB_ALWAYS           = 0x0FFFFFFF,
  CB_INIT             = 0x00000001,
  CB_STEP_ACTIVE      = 0x00000002,
  CB_MAJOR_ITERS      = 0x00000004,
  CB_MINOR_ITERS      = 0x00000008,
  CB_SOLVED           = 0x00000010,
  CB_UPDATE_TIME      = 0x00000020,
  CB_AUGMENT          = 0x00000040,
  CB_STEP_SOLVED      = 0x00000080,
  CB_MATRIX_REFORM    = 0x00000100,
  CB_REMESH           = 0x00000200,
  CB_PRE_MATRIX_SOLVE = 0x00000400,
  CB_RESET            = 0x00000800,
  CB_MODEL_UPDATE     = 0x00001000,
  CB_TIMESTEP_SOLVED  = 0x00002000,
  CB_SERIALIZE_SAVE   = 0x00004000,
  CB_SERIALIZE_LOAD   = 0x00008000,
  CB_TIMESTEP_FAILED  = 0x00010000,
  CB_USER1            = 0x01000000,
};
/* clang-format on */

typedef bool (*CallbackFunction)(FeModel *, uint32_t, void *);

struct Callback {
  CallbackFunction pcb_;  // pointer to callback function
  void *pud_;             // pointer to user data
  uint32_t event_;        // when to call function
};

class CallbackHandler {
 public:
  enum CbInsertPolicy {
    CB_ADD_FRONT,
    CB_ADD_END,
  };

 private:
  Vector<Callback> callbacks_;  //< pointer to callback functions
  uint32_t event_;              //< reason for current callback

 public:
  FECORE_API CallbackHandler();
  FECORE_API ~CallbackHandler();

  void add_callback(CallbackFunction pcb,
                    uint32_t event,
                    void *pud,
                    CbInsertPolicy insert = CB_ADD_END);
};
