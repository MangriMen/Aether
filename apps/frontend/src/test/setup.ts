// jsdom does not implement scrollTo, and the scroll lock in Kobalte calls it on unlock,
// which makes jsdom print "Not implemented" for every dialog or select test
window.scrollTo = () => {};
