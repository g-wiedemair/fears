#include "FeKernel.hpp"

FeKernel *FeKernel::_instance = nullptr;

FeKernel::FeKernel() {}

FeKernel &FeKernel::get_instance() {
  if (_instance == nullptr) {
    _instance = new FeKernel();
    return *_instance;
  }
}
