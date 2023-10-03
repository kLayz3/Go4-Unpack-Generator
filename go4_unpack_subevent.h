#ifndef GO4_UNPACK_SUBEVENT_H
#define GO4_UNPACK_SUBEVENT_H

#include <unordered_map>
#include <cstdint>
#include <cstdio>
	
struct Go4UnpackSubevent {
	uint8_t type;
	uint8_t subtype;
	uint8_t control;
	uint8_t procid;
	uint8_t crate;
	uint8_t subcrate;

	// A reflection-based system
	Go4UnpackSubevent() :
		type(0),
		subtype(0),
		control(0),
		procid(0),
		crate(0),
		subcrate(0) 
	{
		field["type"]     = &this->type;
		field["subtype"]  = &this->subtype;
		field["control"]  = &this->control;
		field["procid"]   = &this->procid;
		field["crate"]    = &this->crate;
		field["subcrate"] = &this->subcrate;
	}

	// Setters and getters via the field hashmap:
	void Set(const char* field_name, uint8_t val) {
		try {
			*field.at(field_name) = val; 
		}
		catch(std::exception& e) {
			printerr("Setting of invalid field \"%s\" in a subevent type.\n", field_name);
		}
	}

	uint8_t Get(const char* field_name) {
		uint8_t x = 0xff;
		try {
			x = *field.at(field_name);
		}
		catch(std::exception& e) {
			printerr("Setting of invalid field \"%s\" in a subevent type.\n", field_name);
		}
		return x;
	}
	
private:
	std::unordered_map<const char*, uint8_t*> field;
};

#endif
