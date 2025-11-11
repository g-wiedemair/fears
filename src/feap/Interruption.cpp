#include "Interruption.hpp"

#include <signal.h>

bool Interruption::bsig_ = false;

Interruption::Interruption() {
  static bool init = false;
  if (!init) {
    signal(SIGINT, Interruption::handler);
    init = true;
  }
}

Interruption::~Interruption() {}

void Interruption::handler(int sig) {
  bsig_ = true;
  signal(SIGINT, Interruption::handler);
}
