function konami(callback) {
    const last10Keys = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // up,up,down,down,left,right,left,right,B,A
    const konami = '38,38,40,40,37,39,37,39,66,65';
    return event => {
        last10Keys.shift();
        last10Keys.push(event.keyCode);
        if (last10Keys.toString() == konami) {
            callback();
        }
    };
}

window.addEventListener('keydown', konami(() => {
    console.log('konami detected');
}));