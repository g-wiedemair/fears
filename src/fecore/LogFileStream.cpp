#include "LogFileStream.hpp"

LogFileStream::~LogFileStream() {
  close();
}

bool LogFileStream::open(const char *filename) {
  if (file_) {
    close();
  }

  filename_ = filename;
  file_ = fopen(filename, "wt");
  return file_ != nullptr;
}

void LogFileStream::close() {
  if (file_)
    fclose(file_);
  file_ = nullptr;
}

void LogFileStream::print(const char *sz) {}

void LogFileStream::flush() {
  fflush(file_);
}
