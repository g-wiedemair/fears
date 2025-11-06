#pragma once

#include "fecore_api.hpp"

class FECORE_API FeKernel {
 private:
  static FeKernel *_instance;

 public:
  static FeKernel &get_instance();

  static void init();
  static void shutdown();

  FeKernel(const FeKernel &) = delete;
  FeKernel &operator=(const FeKernel &) = delete;

  FeKernel();

 private:
  template<typename T, typename... Args> friend T *mem_new(const char *, Args &&...);
};
