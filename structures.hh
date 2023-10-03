#include "go4_unpack_struct.common"

template <typename __T = void> 
struct __BASIC {
  struct {
    unsigned num_items = 0;
    __U32<void> x[4];
    void init() {
      for(int _i = 0; _i < max_dyn; ++_i) data[_i].init();
    }
    bool check_current() {
    }
    void fill(uint8_t* event_handle, size_t& bytes_available, size_t& bytes_read) {
      bytes_read = 0;
      size_t bytes_read_sub;
      while(bytes_available < __U32<void>::min_size() && num_items < 4) {
        data[num_items++].fill(event_handle, bytes_available, bytes_read_sub);
        if(!check_current()) {
          data[--num_items].clear();
          break;
        }
        event_handle += bytes_read_sub;
        bytes_read += bytes_read_sub;
      }
    }
    inline void clear() noexcept {
      for(int _i = 0; _i < num_items; ++_i) data[_i] = 0;
    }
  } x;
  __U32<> y;
  unordered_map<const char*, Go4UnpackPtr> m;

  BASIC() = default;

  void init() {
    {
      void* _p = (void*)&y;
      Go4UnpackPtr _ptr(26, 31, _p);
      m.emplace(std::make_pair("id", _ptr));
    }
  }

  static constexpr size_t min_size() {
    size_t struct_size = 0;
    return struct_size;
  }

  void check_event() {
    bool __b = 1;
    return __b;
  }

  void fill(uint8_t* __event_handle, size_t& bytes_available, size_t& bytes_read) {
    if (min_size() > bytes_available) throw std::runtime_error("Subevent boundary reached. Cannot read anymore.");
    bytes_read = 0;
    size_t bytes_read_sub = 0;
  }

  void clear() {
    x.clear();
    y.clear();
  }

}

