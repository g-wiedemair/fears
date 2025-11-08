#pragma once

#include "fenda/fenda_api.hpp"

class FENDA_API FeapConfig {
 public:
  bool show_errors;

 public:
  FeapConfig();

  void defaults();
};
