#include "FeapModel.hpp"

static LogRef LOG = {"feap.model"};

static bool handle_cb(FeModel *fem, uint32_t event, void *pud) {
  todo();
  return true;
}

FeapModel::FeapModel() {
  log_level_ = LOG_LEVEL_WARN;

  add_callback(handle_cb, CB_ALWAYS, this);
}

LogFile &FeapModel::get_logfile() {
  return logfile_;
}

bool FeapModel::read_input_file(const char *filename) {
  set_input_filename(filename);

  LOG_INFO(&LOG, "Reading input file: %s", input_filename_);

  return 0;
}
