#include "FeapApp.hpp"

int main(int argc, char **argv) {
  // create the feap app
  FeapApp app;

  // initialize the app
  if (app.init(argc, argv) == false)
    return 1;

  // TODO:

  return 1;
}
