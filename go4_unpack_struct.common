#ifndef GO4_UNPACK_STRUCT_COMMON
#define GO4_UNPACK_STRUCT_COMMON

#include <exception>
#include <iostream>
#include <cstdio>

#define MAX_DYN_DEFAULT 128

template<typename _T = void>
struct __D64 {
	uint64_t x;
	__D64() = default;
	static constexpr size_t min_size() {
		return sizeof(uint64_t);
	}
	void init() {}
	inline bool check_event() {return true;}
	inline void clear() noexcept {
		x = 0;
	}
	void fill(uint8_t* event_handle, size_t& bytes_available, size_t& bytes_read) {
		if(min_size() > bytes_available) throw std::runtime_error("Subevent boundary reached. Cannot read anymore.");
		bytes_read = sizeof(uint64_t);
		x = *(uint64_t*)event_handle;
		bytes_available -= bytes_read;
	}
};

// todo!
// copy and write it for __D32, __D16, __D8

// Container for ENCODE ptrs, doesn`t own the pointer to the word
template<typename T>
class Go4UnpackPtr {
	uint64_t* p;
	uint8_t l;
	uint8_t h;
	T mask;
public:
	Go4UnpackPtr() : p(nullptr), l(0), h(0), mask(0) {}
	void assign(void* p, uint8_t l, uint8_t h) {
		this->p = (T*)p;
		this->l = l;
		this->h = h;
		this->mask = ((T)((1ull << (h-l+1)) -1));
		if(h-l+1 > 63) this->mask = (T)0xffffffffffffffff;
	}
	T get_data() const noexcept {
		return mask & (T)(*p >> l);
	}
	T operator*() const noexcept {
		return mask & (T)(*p >> l);
	}
};

// --------------64 bit ----------------//
template<typename _T = void>
using __u64      = __D64<_T>;
template<typename _T = void>
using __i64      = __D64<_T>;
template<typename _T = void>
using __U64      = __D64<_T>;
template<typename _T = void>
using __I64      = __D64<_T>;
template<typename _T = void>
using __uint64_t = __D64<_T>;
template<typename _T = void>
using __int64_t = __D64<_T>;
template<typename _T = void>
using __UInt64_t = __D64<_T>;
template<typename _T = void>
using __Int64_t = __D64<_T>;
using DATA64 = uint64_t;

#endif
