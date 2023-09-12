#pragma once

#include <cmath>

static constexpr double PI = 3.14159265358979323846;
static constexpr double E = 2.71828182845904523536;
static constexpr double TAU = 6.28318530717958647692;

template <typename T> T abs(T n) { return n < 0 ? -n : n; }
template <typename T> T min(T a, T b) { return a < b ? a : b; }
template <typename T> T max(T a, T b) { return a > b ? a : b; }
template <typename T> T clamp(T n, T min, T max) {
    return n < min ? min : n > max ? max : n;
}
template <typename T> T pow(T a, T b) { return std::pow(a, b); }
template <typename T> T sqrt(T n) { return std::sqrt(n); }
template <typename T> T sin(T n) { return std::sin(n); }
template <typename T> T cos(T n) { return std::cos(n); }
template <typename T> T tan(T n) { return std::tan(n); }
template <typename T> T asin(T n) { return std::asin(n); }
template <typename T> T acos(T n) { return std::acos(n); }
template <typename T> T atan(T n) { return std::atan(n); }
template <typename T> T atan2(T y, T x) { return std::atan2(y, x); }
template <typename T> T sinh(T n) { return std::sinh(n); }
template <typename T> T cosh(T n) { return std::cosh(n); }
template <typename T> T tanh(T n) { return std::tanh(n); }
template <typename T> T asinh(T n) { return std::asinh(n); }
template <typename T> T acosh(T n) { return std::acosh(n); }
template <typename T> T atanh(T n) { return std::atanh(n); }
template <typename T> T log(T n) { return std::log(n); }
template <typename T> T log10(T n) { return std::log10(n); }
template <typename T> T log2(T n) { return std::log2(n); }
template <typename T> T exp(T n) { return std::exp(n); }
template <typename T> T exp2(T n) { return std::exp2(n); }
template <typename T> T floor(T n) { return std::floor(n); }
template <typename T> T ceil(T n) { return std::ceil(n); }
template <typename T> T round(T n) { return std::round(n); }
template <typename T> T trunc(T n) { return std::trunc(n); }
template <typename T> T frac(T n) { return n - trunc(n); }
template <typename T> T sign(T n) { return n < 0 ? -1 : n > 0 ? 1 : 0; }
template <typename T> T mod(T a, T b) { return a % b; }
template <typename T> T rem(T a, T b) { return std::remainder(a, b); }
template <typename T> T hypot(T a, T b) { return std::hypot(a, b); }
template <typename T> T cbrt(T n) { return std::cbrt(n); }
template <typename T> T expm1(T n) { return std::expm1(n); }
template <typename T> T log1p(T n) { return std::log1p(n); }
template <typename T> T logb(T n) { return std::logb(n); }
template <typename T> T ilogb(T n) { return std::ilogb(n); }
template <typename T> T lgamma(T n) { return std::lgamma(n); }
