#include "fecore/feap_version.hpp"

#include "core/assert.hpp"
#include "core/string.hpp"
#include "core/utildefines.hpp"
#include <cstring>

static bool version_initialized = false;
static char version_string[50] = "";

static void version_init() {
  const char *version_cycle = "";
  if (STREQ(STRINGIFY(FEAP_VERSION_CYCLE), "alpha")) {
    version_cycle = " Alpha";
  } else if (STREQ(STRINGIFY(FEAP_VERSION_CYCLE), "beta")) {
    version_cycle = " Beta";
  } else if (STREQ(STRINGIFY(FEAP_VERSION_CYCLE), "rc")) {
    version_cycle = " Release Candidate";
  } else if (STREQ(STRINGIFY(FEAP_VERSION_CYCLE), "release")) {
    version_cycle = "";
  } else {
    fassert_msg(0, "Invalid feap version cycle");
  }

  const char *version_suffix = feap_version_is_lts() ? " LTS" : "";

  fsnprintf(version_string,
            sizeof(version_string),
            "%d.%01d.%d%s%s",
            FEAP_VERSION_MAJOR,
            FEAP_VERSION_MINOR,
            FEAP_VERSION_PATCH,
            version_suffix,
            version_cycle);
}

const char *feap_version_string() {
  if (!version_initialized) {
    version_init();
    version_initialized = true;
  }
  return version_string;
}

bool feap_version_is_lts() {
  return STREQ(STRINGIFY(FEAP_VERSION_SUFFIX), "LTS");
}
