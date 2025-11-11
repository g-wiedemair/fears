#pragma once

#include "core/StringRef.hpp"
#include "fecore/FeParam.hpp"
#include "fecore/fecore_api.hpp"

/// A list of material parameters
class FeParamList {
 private:
  FeParamContainer *pc_;
  Vector<FeParam> params_;
  Vector<StringRef> param_groups_;
  int current_group_;

 public:
  FeParamList(FeParamContainer *pc);
  FeParamList(const FeParamList &) = delete;
  FeParamList(FeParamList &&) = delete;
  ~FeParamList();

  FeParamList &operator=(const FeParamList &) = delete;
  FeParamList &operator=(FeParamList &&) = delete;
};

/// Base class for classes that wish to support parameter lists
class FECORE_API FeParamContainer {
 private:
  FeParamList *params_;

 public:
  FeParamContainer();
  virtual ~FeParamContainer();
};
