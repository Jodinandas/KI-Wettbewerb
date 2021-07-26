import logging

class Event:
    """A simple class to implement a (kind of) Observer pattern"""

    def __init__(self, log=True, name=None):
        self._observer_funcs = []
        self._log = log 
        self.name = name if name else self


    def bind(self, *funcs):
        assert min(callable(f) for f in funcs), "Observers must be callable"
        self._observer_funcs.extend(funcs)
        if self._log:
            for func in funcs:
                logging.debug(
                    f"Bound '{func.__name__}' to event '{self.name}'"
                )

    def unbind(self, *funcs):
        for func in funcs:
            try:
                self._observer_funcs.remove(func)
                if self._log:
                    logging.debug(
                        f"Removed binding of '{func.__name__}' to event '{self.name}'"
                    )
            except ValueError:
                continue
    def __iadd__(self, func):
        self.bind(func)
        return self

    def __isub__(self, func):
        self.unbind(func)
        return self
    
    def notify(self, *args, **kwargs):
        """Calls all observer functions"""

        for f in self._observer_funcs:
            f(*args, **kwargs)
            if self._log:
                logging.debug(
                    f"Notified observer '{f.__name__}' of event '{self.name}'"
                )
