from __future__ import annotations




MY_INT_VAR: int = 12
EMPTY: str = """"""
SIMPLE_ASCII: str = """Hello, world!"""
MULTILINE: str = """Line1
Line2
Line3"""
ESCAPED_CHARACTERS: str = """First\\line.
Second "quoted" line.\tEnd."""
UNICODE: str = """Emoji: 😄, Accented: café, Chinese: 世界"""
RAW_STRING: str = r"""Raw \n, "quotes" are okay, and single \ is fine too"""
CONTAINS_BACKTICK: str = """Backtick: ` inside"""
CONTAINS_DOLLAR_CURLY: str = """${not_interpolation}"""
ENDS_WITH_ODD_BACKSLASH: str = r"""Odd number of backslashes: \\""" + '\\'
NULL_BYTE: str = """Null:\x00End"""
COMBINING: str = """é vs é"""
