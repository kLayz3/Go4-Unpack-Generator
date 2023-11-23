#ifndef GO4_UNPACK_SUBEVENT_H
#define GO4_UNPACK_SUBEVENT_H

#include <unordered_map>
#include <cstdint>
#include <cstdio>
#include <string>

#define ADD_FIELD(T,x) \
	private: T x; \
	public: T get_##x () { return x; } \
	void set_##x (T val) { x = val; }

struct Go4UnpackSubevent {
	uint16_t subtype;
	uint16_t i_type;
	uint16_t h_control;
	uint16_t h_subcrate;
	uint16_t i_procid;

	Go4UnpackSubevent() :
		l_dlen(0),
		i_subtype(0xffff),
		i_type(0xffff),
		h_control(0xff),
		h_subcrate(0xff),
		i_procid(0xffff) {}

	Go4UnpackSubevent(TGo4Mbs
	virtual void init() = 0;
	virtual void fill(uint8_t* subev_ptr, size_t&)
	virtual void check_event() = 0;
	virtual void clear() = 0;
};

#endif
