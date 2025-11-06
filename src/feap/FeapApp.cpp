#include "FeapApp.hpp"

#include "fecore/FeKernel.hpp"

FeapApp::FeapApp() {}

bool FeapApp::init(int, char **) {
  // initialize kernel
  FeKernel::init();

  return true;
}

void FeapApp::finish() {
  FeKernel::shutdown();
}
