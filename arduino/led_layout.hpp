#pragma once

// Layout of leds is
//
// ***********************
// *                     *
// *                     *
// *       Screen        *
// *                     *
// *                     *
// ****E             S****
//
// All are connected into single stripe and addressed counter clockwise.

#include <cstdint>

constexpr uintptr_t LedsBottomRight = 6;
constexpr uintptr_t LedsRight = 19;
constexpr uintptr_t LedsTop = 35;
constexpr uintptr_t LedsLeft = 19;
constexpr uintptr_t LedsBottomLeft = 6;
constexpr uintptr_t LedsTotal =
    LedsBottomRight + LedsRight + LedsTop + LedsLeft + LedsBottomLeft;

// serial data is sent as |Header|RGBRGBRGB....|Footer|
constexpr uint8_t const SerialDataHeader[] = "WAMB";
constexpr uint8_t const SerialDataFooter[] = "BMAW";
