#include "FeapApp.hpp"

#include "fecore/FeKernel.hpp"

FeapApp::FeapApp() {}

bool FeapApp::init(int argc, char **argv) {
  // initialize kernel
  FeKernel::init();
}
