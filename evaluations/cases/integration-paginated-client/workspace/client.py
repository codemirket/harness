"""Implement the loopback catalog protocol described in protocol.md."""


class MutationUncertain(Exception):
    """A write may have been accepted but no response was confirmed."""


def list_all(base_url, token):
    raise NotImplementedError


def create_item(base_url, token, name):
    raise NotImplementedError
