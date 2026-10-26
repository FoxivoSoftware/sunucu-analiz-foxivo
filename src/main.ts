import { mount } from 'svelte'
import '@fontsource-variable/inter/opsz.css'
import './app.css'
import App from './App.svelte'

export default mount(App, { target: document.getElementById('app')! })
