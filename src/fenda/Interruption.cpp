#include "Interruption.hpp"

#include <signal.h>

bool Interruption::m_bsig = false;

Interruption::Interruption() {
  static bool init = false;
  if (!init) {
    signal(SIGINT, Interruption::handler);
    init = true;
  }
}

Interruption::~Interruption() {}

void Interruption::handler(int sig) {
  m_bsig = true;
  signal(SIGINT, Interruption::handler);
}
