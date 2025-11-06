#include "FeKernel.hpp"

#include "core/memory.hpp"

FeKernel *FeKernel::_instance = nullptr;

FeKernel::FeKernel() {}

FeKernel &FeKernel::get_instance() {
  if (_instance == nullptr) {
    _instance = mem_new<FeKernel>(__func__);
  }
  return *_instance;
}

void FeKernel::init() {
  if (_instance == nullptr) {
    _instance = mem_new<FeKernel>(__func__);
  }
}

void FeKernel::shutdown() {
  mem_delete(_instance);
  _instance = nullptr;
}
