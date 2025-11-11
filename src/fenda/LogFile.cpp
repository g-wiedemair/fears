#include "LogFile.hpp"

#include "core/memory.hpp"

LogFile::LogFile() {
  console_ = nullptr;
  file_ = nullptr;
  mode_ = LOG_FILE_AND_CONSOLE;
}

LogFile::~LogFile() {
  close();

  if (console_) {
    mem_delete(console_);
    console_ = nullptr;
  }
}

bool LogFile::open(const char *filename) {
  if (file_ == nullptr) {
    file_ = mem_new<LogFileStream>(__func__);
  }
  return file_->open(filename);
}

void LogFile::close() {
  if (file_) {
    file_->close();
    mem_delete(file_);
    file_ = nullptr;
  }
}
