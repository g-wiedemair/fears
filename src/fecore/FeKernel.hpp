#pragma once

class FeKernel {
private:
  static FeKernel *_instance;

public:
  FeKernel &get_instance();

  FeKernel(const FeKernel &) = delete;
  FeKernel &operator=(const FeKernel &) = delete;

private:
  FeKernel();
};
