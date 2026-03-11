describe('sanity checks', () => {
  it('basic math works', () => {
    expect(1 + 1).toBe(2);
  });

  it('app element exists in DOM', () => {
    document.body.innerHTML = '<div id="app"></div>';
    const app = document.getElementById('app');
    expect(app).not.toBeNull();
    expect(app!.tagName).toBe('DIV');
  });
});
