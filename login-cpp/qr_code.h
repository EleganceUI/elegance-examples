#ifndef LOGIN_QR_CODE_H
#define LOGIN_QR_CODE_H

#include <elg/elegance.h>

#include <string>

// `text` as a QR code (high error correction, so a logo can cover the middle), drawn with a
// Canvas: dark modules on a transparent background, scaled to whole points.
elg::View QrCode(const std::string& text, elg::Color color = elg::Color::black);

#endif // LOGIN_QR_CODE_H
