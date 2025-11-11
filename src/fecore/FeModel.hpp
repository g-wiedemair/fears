#pragma once

#include "fecore/Callback.hpp"
#include "fecore/FeCoreBase.hpp"
#include "fecore/fecore_api.hpp"

/// The FeModel class stores all the data for the finite element model, including
/// geometry, analysis steps, boundary and loading conditions, contact interfaces
/// and so on
class FeModel : public FeCoreBase, public CallbackHandler {
 public:
  FeModel() = default;
  FeModel(const FeModel &) = delete;
  FeModel(FeModel &&) = delete;
  ~FeModel() override = default;
};
