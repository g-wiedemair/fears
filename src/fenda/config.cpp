#include "core/map.hpp"
#include "core/string.hpp"
#include "fenda.hpp"

#include "fenda/FeapConfig.hpp"

static HashMap<String, String> vars;
static bool boutput = true;

//-------------------------------------------------------------------------------------------------
// configure Fenda

bool fenda::configure(const char *file, FeapConfig &config) {
  return true;
}
