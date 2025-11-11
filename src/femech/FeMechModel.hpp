#pragma once

#include "fecore/FeModel.hpp"

/// This class extends the basic FEModel class by adding a rigid body system
class FeMechModel : public FeModel {
 public:
  FeMechModel() = default;
  FeMechModel(const FeMechModel &) = delete;
  FeMechModel(FeMechModel &&) = delete;
  ~FeMechModel() override = default;
};
