#pragma once

#include "core/Span.hpp"
#include "core/string.hpp"
#include <string_view>

/**
 * A common base class for StringRef and StringRefNull.
 * This should never be used in other files. It only exists to avoid some code duplication
 */
class StringRefBase {
 protected:
  const char *data_;
  int64_t size_;

  constexpr StringRefBase(const char *data, int64_t size);

 public:
  static constexpr int64_t not_found = -1;

  operator String() const;
  constexpr operator std::string_view() const;

  constexpr const char *begin() const;
  constexpr const char *end() const;
};

//-------------------------------------------------------------------------------------------------
// StringRefBase inline methods

constexpr StringRefBase::StringRefBase(const char *data, const int64_t size)
    : data_(data), size_(size) {}

inline StringRefBase::operator String() const {
  return String(data_, size_t(size_));
}

constexpr StringRefBase::operator std::string_view() const {
  return std::string_view(data_, size_t(size_));
}

constexpr const char *StringRefBase::begin() const {
  return data_;
}

constexpr const char *StringRefBase::end() const {
  return data_ + size_;
}

//-------------------------------------------------------------------------------------------------

/**
 * References a const char array. It might not be null terminated
 * StringRef can be compared with StringRef and StringRefNull
 */
class StringRef : public StringRefBase {
 public:
  constexpr StringRef();
  constexpr StringRef(const char *str);
  constexpr StringRef(const char *str, int64_t length);
  constexpr StringRef(const char *begin, const char *one_after_end);
  constexpr StringRef(std::string_view view);
  constexpr StringRef(Span<char> span);
  StringRef(const String &str);

  constexpr char operator[](int64_t index) const;
};

//-------------------------------------------------------------------------------------------------
// StringRef inline methods

constexpr StringRef::StringRef() : StringRefBase(nullptr, 0) {}

constexpr StringRef::StringRef(const char *str)
    : StringRefBase(str, str ? int64_t(std::char_traits<char>::length(str)) : 0) {}

//-------------------------------------------------------------------------------------------------
// Operator Overloads

constexpr bool operator==(StringRef a, StringRef b) {
  return std::string_view(a) == std::string_view(b);
}

constexpr bool operator!=(StringRef a, StringRef b) {
  return std::string_view(a) != std::string_view(b);
}
