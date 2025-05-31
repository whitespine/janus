// shim.js
const { core } = Deno;

function argsToMessage(...args) {
    return args.map((arg) => JSON.stringify(arg)).join(" ");
}

globalThis.console = {
    log: (...args) => {
        core.print(`[out]: ${argsToMessage(...args)}\n`, false);
    },
    warn: (...args) => {
        core.print(`[warn]: ${argsToMessage(...args)}\n`, true);
    },
    error: (...args) => {
        core.print(`[err]: ${argsToMessage(...args)}\n`, true);
    },
};

globalThis.logger = console;
