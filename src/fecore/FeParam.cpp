#include "FeParam.hpp"

FeParam::FeParam(void *pdata, FeParamType type, int dim, const char *name, bool *watch)
    : data_(pdata),
      type_(type),
      dim_(dim),
      name_(name),
      long_name_(name),
      watch_(watch),
      unit_(nullptr),
      enum_(nullptr),
      group_(-1),
      parent_(nullptr),
      validator_(nullptr) {
  if (watch_) {
    watch = false;
  }

  // Default flags
  flags_ = 0;
  if (dim_ == 1) {
    switch (type_) {
      case FE_PARAM_FLOAT:
      case FE_PARAM_VEC3:
      case FE_PARAM_FLOAT_MAPPED:
      case FE_PARAM_VEC3_MAPPED:
        flags_ = FE_PARAM_VOLATILE;
        break;
    }
  }
}

FeParam::~FeParam() {
  if (flags_ & FE_PARAM_USER) {
    todo();
  }
}

//-------------------------------------------------------------------------------------------------
