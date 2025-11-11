#pragma once

#include "core/Vector.hpp"
#include "fecore/fecore_api.hpp"

class FeParamContainer;
class FeParamValidator;

/// Different supported parameter types
enum FeParamType {
  FE_PARAM_INVALID,
  FE_PARAM_INT,
  FE_PARAM_BOOL,
  FE_PARAM_FLOAT,
  FE_PARAM_VEC2,
  FE_PARAM_VEC3,
  FE_PARAM_MAT3,
  FE_PARAM_STRING,
  FE_PARAM_DATA_ARRAY,
  FE_PARAM_FLOAT_MAPPED,
  FE_PARAM_VEC3_MAPPED,
  FE_PARAM_MATERIALPOINT,
};

/* clang-format off */
enum FeParamFlags : uint32_t {
  FE_PARAM_ATTRIBUTE = 0x01,  // parameter will be read as attribute
  FE_PARAM_USER      = 0x02,  // user parameter (owned by parameter list)
  FE_PARAM_HIDDEN    = 0x04,  // hides parameters (in FeView)
  FE_PARAM_ADDLC     = 0x08,  // parameter should get a default load curve
  FE_PARAM_VOLATILE  = 0x10,  // parameter can change (e.g. via a load curve)
  FE_PARAM_TOPLEVEL  = 0x20,  // parameter should only be defined at top-level (materials only)
  FE_PARAM_WATCH     = 0x40,  // watch parameter
  FE_PARAM_OBSOLETE  = 0x80,  // parameter is obsolete
};
/* clang-format on */

/// This class describes a user-defined parameter
class FECORE_API FeParam {
 private:
  void *data_;                   // pointer to variable data
  int dim_;                      // dimension (in case data is array)
  FeParamType type_;             // type of variable
  uint32_t flags_;               // parameter flags
  bool *watch_;                  // parameter watch (set to true if read in)
  int group_;                    // index of parameter group (-1 by default)
                                 //
  const char *name_;             // name of the parameter
  const char *enum_;             // enumerate values for ints
  const char *unit_;             // unit string
  const char *long_name_;        // more descriptive name (optional)
                                 //
  FeParamValidator *validator_;  //
  FeParamContainer *parent_;     // parent object of parameter

 public:
  FeParam(void *pdata, FeParamType type, int dim, const char *name, bool *watch = nullptr);
  FeParam(const FeParam &) = delete;
  FeParam(FeParam &&) = delete;
  ~FeParam();

  FeParam &operator=(const FeParam &) = delete;
  FeParam &operator=(FeParam &&) = delete;
};
