/* SPDX-License-Identifier: GPL-2.0-or-later */
/*
 * Ferrophone build settings for pjproject. Copied to
 * pjlib/include/pj/config_site.h by scripts/build-pjproject.sh.
 * Every consumer of the built libraries must see the same values.
 *
 * Runtime choices (100rel, echo canceller type, SRTP mode) are set per
 * account or media config by ferro-engine, not here. AEC3 is compiled in
 * by configure's --enable-libwebrtc-aec3.
 */
#ifndef FERROPHONE_CONFIG_SITE_H
#define FERROPHONE_CONFIG_SITE_H

/* FR-01: at least 8 accounts; headroom for test rigs. */
#define PJSUA_MAX_ACC 32

/* FR-06: at least 4 line appearances; headroom for conference and transfer legs. */
#define PJSUA_MAX_CALLS 32

/* 2 sockets (RTP + RTCP) per call, plus SIP transports and DNS. */
#define PJ_IOQUEUE_MAX_HANDLES 256

/* FR-14: BLF grid and sidecar. One buddy per BLF key. */
#define PJSUA_MAX_BUDDIES 512

/* FR-12, FR-38: one player and one recorder per call, plus ringers and test hooks. */
#define PJSUA_MAX_PLAYERS 64
#define PJSUA_MAX_RECORDERS 64

/* Conference bridge slots: calls + players + recorders + sound device. */
#define PJSUA_MAX_CONF_PORTS 192

/* D-09: audio only for 1.0. */
#define PJMEDIA_HAS_VIDEO 0

/* NFR-04: 10 ms audio frames for low latency. */
#define PJSUA_DEFAULT_AUDIO_FRAME_PTIME 10
#define PJSUA_DEFAULT_EC_TAIL_LEN 200

/* FR-23: SRTP with SDES. */
#define PJMEDIA_HAS_SRTP 1

/* FR-02: DNS SRV/NAPTR through PJSIP's own resolver. */
#define PJSIP_HAS_RESOLVER 1

#endif
