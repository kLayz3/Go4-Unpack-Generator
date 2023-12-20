#ifndef __GO4_UNPACK_SUBEVENT_H__
#define __GO4_UNPACK_SUBEVENT_H__

#include <unordered_map>
#include <cstdint>
#include <cstdio>
#include <string>

#define ADD_FIELD(T,x) \
	private: T x; \
	public: T get_##x () { return x; } \
	void set_##x (T val) { x = val; }

class Go4UnpackBaseSubevent {
	const uint16_t type;
	const uint16_t subtype;
	const uint16_t procid;
	const uint16_t subcrate;
	const uint16_t control;
public:
	uint32_t l_dlen;	
	Go4UnpackSubevent() = default;
	
	virtual void init() = 0;
	virtual void fill(uint8_t*, size_t&) = 0;
	virtual void check_event() = 0;
	virtual void clear() = 0;
};

#endif
