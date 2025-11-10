#pragma once

#include "fenda/fenda_api.hpp"

class FENDA_API Interruption {
 public:
  Interruption();
  virtual ~Interruption();

  static void handler(int sig);
  static bool m_bsig;
};
