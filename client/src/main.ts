const app = document.getElementById('app')!;

async function init() {
  try {
    const response = await fetch('/api/ping');
    const data = await response.json();
    app.textContent = `Server status: ${data.status}`;
    console.log('Server ping:', data);
  } catch {
    app.textContent = 'Real Micro Machines - Server not connected';
    console.log('Server not available, running in standalone mode');
  }
}

init();
