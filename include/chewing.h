/*
 * Copyright (c) 2024
 *      libchewing Core Team.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef chewing_public_bindings_h
#define chewing_public_bindings_h

#pragma once

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/** @brief context handle used for Chewing IM APIs
 */
typedef struct ChewingContext ChewingContext;

/** @brief indicate the internal encoding of data processing.
 *  @since 0.3.0
 */
#define LIBCHEWING_ENCODING "UTF-8"



/**
 * Indicates chewing will translate keystrokes to Chinese characters.
 */
#define CHINESE_MODE 1

/**
 * Indicates the input mode is translating keystrokes to symbols.
 */
#define SYMBOL_MODE 0

/**
 * Indicates chewing will translate latin and puctuation characters to
 * double-with characters.
 */
#define FULLSHAPE_MODE 1

/**
 * Indicates chewing will not translate latin and puctuation characters.
 */
#define HALFSHAPE_MODE 0

/**
 * Use conversion engine that doesn't perform intelligent phrasing.
 */
#define SIMPLE_CONVERSION_ENGINE 0

/**
 * Use the original Chewing intelligent phrasing.
 */
#define CHEWING_CONVERSION_ENGINE 1

/**
 * Use original Chewing intelligent phrasing with fuzzy prefix search.
 */
#define FUZZY_CHEWING_CONVERSION_ENGINE 2

/**
 * Indicates automatic user phrase learning is disabled.
 */
#define AUTOLEARN_DISABLED 1

/**
 * Indicates automatic user phrase learning is enabled.
 */
#define AUTOLEARN_ENABLED 0

/**
 * The minimal size of pre-edit buffer.
 */
#define MIN_CHI_SYMBOL_LEN 0

/**
 * The maximum size of pre-edit buffer.
 */
#define MAX_CHI_SYMBOL_LEN (MAX_PHONE_SEQ_LEN - MAX_PHRASE_LEN)

/**
 * The size of internal buffer for pre-edit buffer.
 */
#define MAX_PHONE_SEQ_LEN 50

/**
 * The maximum phrase size.
 */
#define MAX_PHRASE_LEN 11

/**
 * The number of minimum candidates that are selectable via shortcut keys.
 */
#define MIN_SELKEY 1

/**
 * The number of maximum candidates that are selectable via shortcut keys.
 */
#define MAX_SELKEY 10

/**
 * Log level.
 */
#define CHEWING_LOG_VERBOSE 1

/**
 * Log level.
 */
#define CHEWING_LOG_DEBUG 2

/**
 * Log level.
 */
#define CHEWING_LOG_INFO 3

/**
 * Log level.
 */
#define CHEWING_LOG_WARN 4

/**
 * Log level.
 */
#define CHEWING_LOG_ERROR 5

/**
 * Use "asdfjkl789" as selection key.
 */
#define HSU_SELKEY_TYPE1 1

/**
 * Use "asdfzxcv89" as selection key.
 */
#define HSU_SELKEY_TYPE2 2

#define KEYSTROKE_IGNORE 1

#define KEYSTROKE_COMMIT 2

#define KEYSTROKE_BELL 4

#define KEYSTROKE_ABSORB 8

#define CHEWING_VERSION_MAJOR 0

#define CHEWING_VERSION_MINOR 9

#define CHEWING_VERSION_PATCH 1

/**
 * Keyboard layout index.
 *
 */
typedef enum KB {
  KB_DEFAULT,
  KB_HSU,
  KB_IBM,
  KB_GIN_YIEH,
  KB_ET,
  KB_ET26,
  KB_DVORAK,
  KB_DVORAK_HSU,
  KB_DACHEN_CP26,
  KB_HANYU_PINYIN,
  KB_THL_PINYIN,
  KB_MPS2_PINYIN,
  KB_CARPALX,
  KB_COLEMAK_DH_ANSI,
  KB_COLEMAK_DH_ORTH,
  KB_WORKMAN,
  KB_COLEMAK,
  KB_TYPE_NUM,
} KB;

/**
 * Opaque context handle used for chewing APIs.
 *
 */
typedef struct ChewingContext ChewingContext;

/**
 * Specifies the interval of a phrase segment in the pre-editng area
 */
typedef struct IntervalType {
  /**
   * Starting position of certain interval
   */
  int from;
  /**
   * Ending position of certain interval (exclusive)
   */
  int to;
} IntervalType;

/**
 * Configuration for chewing runtime features.
 *
 * Deprecated, use chewing_set_ series of functions to set parameters instead.
 *
 */
typedef struct ChewingConfigData {
  int candPerPage;
  int maxChiSymbolLen;
  int selKey[MAX_SELKEY];
  int bAddPhraseForward;
  int bSpaceAsSelection;
  int bEscCleanAllBuf;
  int bAutoShiftCur;
  int bEasySymbolInput;
  int bPhraseChoiceRearward;
  int hsuSelKeyType;
} ChewingConfigData;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct ChewingContext *chewing_new(void);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
struct ChewingContext *chewing_new2(const char *syspath,
                                    const char *userpath,
                                    void (*logger)(void *data, int level, const char *fmt, ...),
                                    void *loggerdata);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_delete(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_free(void *ptr);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_Reset(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_ack(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_config_has_option(const struct ChewingContext *ctx, const char *name);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_config_get_int(const struct ChewingContext *ctx, const char *name);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_config_set_int(struct ChewingContext *ctx, const char *name, int value);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_config_get_str(const struct ChewingContext *ctx, const char *name, char **value);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_config_set_str(struct ChewingContext *ctx, const char *name, const char *value);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_set_KBType(struct ChewingContext *ctx, int kbtype);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_KBType(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_get_KBString(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_KBStr2Num(const char *str);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_ChiEngMode(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_ChiEngMode(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_ShapeMode(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_ShapeMode(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_candPerPage(struct ChewingContext *ctx, int n);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_candPerPage(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_maxChiSymbolLen(struct ChewingContext *ctx, int n);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_maxChiSymbolLen(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_selKey(struct ChewingContext *ctx, const int *sel_keys, int len);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int *chewing_get_selKey(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_addPhraseDirection(struct ChewingContext *ctx, int direction);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_addPhraseDirection(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_spaceAsSelection(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_spaceAsSelection(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_escCleanAllBuf(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_escCleanAllBuf(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_autoShiftCur(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_autoShiftCur(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_easySymbolInput(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_easySymbolInput(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_phraseChoiceRearward(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_phraseChoiceRearward(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_autoLearn(struct ChewingContext *ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_autoLearn(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
unsigned short *chewing_get_phoneSeq(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_phoneSeqLen(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_logger(struct ChewingContext *ctx, void (*logger)(void *data,
                                                                   int level,
                                                                   const char *fmt,
                                                                   ...), void *data);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_userphrase_enumerate(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_userphrase_has_next(struct ChewingContext *ctx,
                                unsigned int *phrase_len,
                                unsigned int *bopomofo_len);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_userphrase_get(struct ChewingContext *ctx,
                           char *phrase_buf,
                           unsigned int phrase_len,
                           char *bopomofo_buf,
                           unsigned int bopomofo_len);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_userphrase_add(struct ChewingContext *ctx,
                           const char *phrase_buf,
                           const char *bopomofo_buf);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_userphrase_remove(struct ChewingContext *ctx,
                              const char *phrase_buf,
                              const char *bopomofo_buf);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_userphrase_lookup(struct ChewingContext *ctx,
                              const char *phrase_buf,
                              const char *bopomofo_buf);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_list_first(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_list_last(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_list_has_next(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_list_has_prev(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_list_next(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_list_prev(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_commit_preedit_buf(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_clean_preedit_buf(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_clean_bopomofo_buf(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_phone_to_bopomofo(unsigned short phone, char *buf, unsigned short len);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Space(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Esc(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Enter(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Del(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Backspace(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Tab(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_ShiftLeft(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Left(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_ShiftRight(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Right(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Up(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Home(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_End(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_PageUp(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_PageDown(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Down(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Capslock(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Default(struct ChewingContext *ctx, int key);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_CtrlNum(struct ChewingContext *ctx, int key);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_ShiftSpace(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_DblTab(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_handle_Numlock(struct ChewingContext *ctx, int key);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_commit_Check(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_commit_String(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_commit_String_static(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_buffer_String(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_buffer_String_static(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_buffer_Check(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_buffer_Len(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_bopomofo_String_static(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_bopomofo_String(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_bopomofo_Check(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cursor_Current(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_CheckDone(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_TotalPage(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_ChoicePerPage(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_TotalChoice(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_CurrentPage(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_cand_Enumerate(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_hasNext(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_cand_String(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_cand_String_static(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_cand_string_by_index(struct ChewingContext *ctx, int index);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_cand_string_by_index_static(struct ChewingContext *ctx, int index);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_choose_by_index(struct ChewingContext *ctx, int index);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_open(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_cand_close(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_interval_Enumerate(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_interval_hasNext(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_interval_Get(struct ChewingContext *ctx, struct IntervalType *it);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_aux_Check(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_aux_Length(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_aux_String(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_aux_String_static(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_keystroke_CheckIgnore(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_keystroke_CheckAbsorb(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_kbtype_Total(const struct ChewingContext *_ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_kbtype_Enumerate(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_kbtype_hasNext(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_kbtype_String(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
const char *chewing_kbtype_String_static(struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_zuin_Check(const struct ChewingContext *ctx);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
char *chewing_zuin_String(const struct ChewingContext *ctx, int *zuin_count);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_Init(const char *data_path, const char *hash_path);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_Terminate(void);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_Configure(struct ChewingContext *ctx, struct ChewingConfigData *pcd);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
void chewing_set_hsuSelKeyType(struct ChewingContext *_ctx, int mode);

/**
 * # Safety
 *
 * This function should be called with valid pointers.
 */
int chewing_get_hsuSelKeyType(struct ChewingContext *_ctx);

const char *chewing_version(void);

int chewing_version_major(void);

int chewing_version_minor(void);

int chewing_version_patch(void);

const char *chewing_version_extra(void);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* chewing_public_bindings_h */
