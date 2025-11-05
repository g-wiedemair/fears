//
// Created by wig on 05.11.2025.
//

#pragma once

class FeapApp {
private:
public:
  FeapApp();

  bool init(int argc, char **argv);

public:
  FeapApp(const FeapApp &) = delete;
  FeapApp &operator=(const FeapApp &) = delete;
};
