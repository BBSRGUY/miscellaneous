/**
 * Visual Engine using Three.js
 * Handles 3D piano rendering and animations
 */

class VisualEngine {
    constructor(containerElement) {
        this.container = containerElement;
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.pianoKeys = new Map();
        this.particles = [];
        this.style = 'neon'; // 'neon', 'classic', 'particles'

        // Piano configuration
        this.whiteKeyWidth = 0.8;
        this.whiteKeyLength = 5;
        this.whiteKeyHeight = 0.3;
        this.blackKeyWidth = 0.5;
        this.blackKeyLength = 3;
        this.blackKeyHeight = 0.6;

        // Animation properties
        this.clock = new THREE.Clock();
        this.activeAnimations = new Map();

        this.init();
    }

    /**
     * Initialize Three.js scene
     */
    init() {
        // Create scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x000000);
        this.scene.fog = new THREE.Fog(0x000000, 10, 100);

        // Create camera
        this.camera = new THREE.PerspectiveCamera(
            60,
            window.innerWidth / window.innerHeight,
            0.1,
            1000
        );
        this.camera.position.set(0, 12, 15);
        this.camera.lookAt(0, 0, 0);

        // Create renderer
        this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
        this.renderer.setSize(window.innerWidth, window.innerHeight);
        this.renderer.setPixelRatio(window.devicePixelRatio);
        this.container.appendChild(this.renderer.domElement);

        // Add lights
        this.setupLights();

        // Create the piano
        this.createPiano();

        // Add environment
        this.createEnvironment();

        // Handle window resize
        window.addEventListener('resize', () => this.onWindowResize());

        // Start animation loop
        this.animate();

        console.log('Visual Engine initialized');
    }

    /**
     * Setup scene lighting
     */
    setupLights() {
        // Ambient light for base illumination
        const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
        this.scene.add(ambientLight);

        // Main directional light
        const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
        dirLight.position.set(5, 10, 5);
        dirLight.castShadow = true;
        this.scene.add(dirLight);

        // Colored point lights for neon effect
        const pinkLight = new THREE.PointLight(0xff00ff, 2, 30);
        pinkLight.position.set(-10, 5, 5);
        this.scene.add(pinkLight);

        const cyanLight = new THREE.PointLight(0x00ffff, 2, 30);
        cyanLight.position.set(10, 5, 5);
        this.scene.add(cyanLight);

        // Store lights for style changes
        this.lights = {
            ambient: ambientLight,
            directional: dirLight,
            pink: pinkLight,
            cyan: cyanLight
        };
    }

    /**
     * Create the 3D piano
     */
    createPiano() {
        const pianoGroup = new THREE.Group();

        // Piano key layout (note names for 2 octaves starting from C)
        const whiteKeys = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
        const blackKeyPositions = [0.5, 1.5, 3.5, 4.5, 5.5]; // Positions relative to white keys

        let whiteKeyIndex = 0;

        // Create white keys (2 octaves)
        for (let octave = 0; octave < 2; octave++) {
            for (let i = 0; i < whiteKeys.length; i++) {
                const noteName = whiteKeys[i];
                const keyMesh = this.createWhiteKey(whiteKeyIndex);
                const fullNoteName = `${noteName}${4 + octave}`;

                this.pianoKeys.set(fullNoteName, {
                    mesh: keyMesh,
                    isBlack: false,
                    baseY: 0,
                    pressedY: -0.2
                });

                pianoGroup.add(keyMesh);
                whiteKeyIndex++;
            }
        }

        // Create black keys
        const blackKeys = ['C#', 'D#', 'F#', 'G#', 'A#'];
        for (let octave = 0; octave < 2; octave++) {
            for (let i = 0; i < blackKeyPositions.length; i++) {
                const position = blackKeyPositions[i] + (octave * 7);
                const noteName = blackKeys[i];
                const keyMesh = this.createBlackKey(position);
                const fullNoteName = `${noteName}${4 + octave}`;

                this.pianoKeys.set(fullNoteName, {
                    mesh: keyMesh,
                    isBlack: true,
                    baseY: this.whiteKeyHeight / 2,
                    pressedY: this.whiteKeyHeight / 2 - 0.15
                });

                pianoGroup.add(keyMesh);
            }
        }

        // Center the piano
        pianoGroup.position.x = -5.6;

        this.scene.add(pianoGroup);
        this.pianoGroup = pianoGroup;
    }

    /**
     * Create a white piano key
     */
    createWhiteKey(index) {
        const geometry = new THREE.BoxGeometry(
            this.whiteKeyWidth,
            this.whiteKeyHeight,
            this.whiteKeyLength
        );

        const material = new THREE.MeshStandardMaterial({
            color: 0xffffff,
            emissive: 0x000000,
            metalness: 0.3,
            roughness: 0.4
        });

        const key = new THREE.Mesh(geometry, material);
        key.position.x = index * this.whiteKeyWidth;
        key.position.y = 0;
        key.position.z = 0;

        return key;
    }

    /**
     * Create a black piano key
     */
    createBlackKey(position) {
        const geometry = new THREE.BoxGeometry(
            this.blackKeyWidth,
            this.blackKeyHeight,
            this.blackKeyLength
        );

        const material = new THREE.MeshStandardMaterial({
            color: 0x000000,
            emissive: 0x000000,
            metalness: 0.5,
            roughness: 0.3
        });

        const key = new THREE.Mesh(geometry, material);
        key.position.x = position * this.whiteKeyWidth;
        key.position.y = this.whiteKeyHeight / 2;
        key.position.z = -1;

        return key;
    }

    /**
     * Create the environment (grid, etc.)
     */
    createEnvironment() {
        // Create grid for neon style
        const gridSize = 50;
        const gridDivisions = 50;
        const gridHelper = new THREE.GridHelper(gridSize, gridDivisions, 0xff00ff, 0x00ffff);
        gridHelper.position.y = -2;
        gridHelper.material.opacity = 0.3;
        gridHelper.material.transparent = true;
        this.scene.add(gridHelper);
        this.gridHelper = gridHelper;

        // Add particles
        this.createParticleSystem();
    }

    /**
     * Create particle system for visual effects
     */
    createParticleSystem() {
        const particleCount = 100;
        const geometry = new THREE.BufferGeometry();
        const positions = new Float32Array(particleCount * 3);
        const colors = new Float32Array(particleCount * 3);

        for (let i = 0; i < particleCount; i++) {
            const i3 = i * 3;
            positions[i3] = (Math.random() - 0.5) * 30;
            positions[i3 + 1] = Math.random() * 20;
            positions[i3 + 2] = (Math.random() - 0.5) * 30;

            colors[i3] = Math.random();
            colors[i3 + 1] = Math.random();
            colors[i3 + 2] = Math.random();
        }

        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

        const material = new THREE.PointsMaterial({
            size: 0.1,
            vertexColors: true,
            transparent: true,
            opacity: 0.6
        });

        this.particleSystem = new THREE.Points(geometry, material);
        this.scene.add(this.particleSystem);
    }

    /**
     * Animate a key press
     */
    pressKey(noteName) {
        const keyData = this.pianoKeys.get(noteName);
        if (!keyData) return;

        const { mesh, pressedY } = keyData;

        // Animate key down
        const startY = mesh.position.y;
        const duration = 50; // milliseconds
        const startTime = Date.now();

        const animate = () => {
            const elapsed = Date.now() - startTime;
            const progress = Math.min(elapsed / duration, 1);

            mesh.position.y = startY + (pressedY - startY) * progress;

            // Update emissive color for glow effect
            if (mesh.material) {
                const glowIntensity = progress;
                if (keyData.isBlack) {
                    mesh.material.emissive.setHex(0xff00ff);
                    mesh.material.emissiveIntensity = glowIntensity;
                } else {
                    mesh.material.emissive.setHex(0x00ffff);
                    mesh.material.emissiveIntensity = glowIntensity * 0.5;
                }
            }

            if (progress < 1) {
                requestAnimationFrame(animate);
            }
        };

        animate();

        // Create particle burst if in particles mode
        if (this.style === 'particles') {
            this.createParticleBurst(mesh.position);
        }
    }

    /**
     * Animate a key release
     */
    releaseKey(noteName) {
        const keyData = this.pianoKeys.get(noteName);
        if (!keyData) return;

        const { mesh, baseY } = keyData;

        // Animate key up
        const startY = mesh.position.y;
        const duration = 100; // milliseconds
        const startTime = Date.now();

        const animate = () => {
            const elapsed = Date.now() - startTime;
            const progress = Math.min(elapsed / duration, 1);

            mesh.position.y = startY + (baseY - startY) * progress;

            // Fade out emissive glow
            if (mesh.material) {
                const glowIntensity = 1 - progress;
                mesh.material.emissiveIntensity = glowIntensity;
            }

            if (progress < 1) {
                requestAnimationFrame(animate);
            } else {
                // Reset emissive
                if (mesh.material) {
                    mesh.material.emissive.setHex(0x000000);
                    mesh.material.emissiveIntensity = 0;
                }
            }
        };

        animate();
    }

    /**
     * Create a particle burst effect
     */
    createParticleBurst(position) {
        const particleCount = 20;
        const particles = [];

        for (let i = 0; i < particleCount; i++) {
            const geometry = new THREE.SphereGeometry(0.1, 8, 8);
            const material = new THREE.MeshBasicMaterial({
                color: Math.random() > 0.5 ? 0xff00ff : 0x00ffff,
                transparent: true,
                opacity: 1
            });

            const particle = new THREE.Mesh(geometry, material);
            particle.position.copy(position);
            particle.position.y += 1;

            // Random velocity
            particle.velocity = new THREE.Vector3(
                (Math.random() - 0.5) * 0.2,
                Math.random() * 0.3,
                (Math.random() - 0.5) * 0.2
            );

            particle.life = 1.0;

            this.scene.add(particle);
            particles.push(particle);
        }

        this.particles.push(...particles);
    }

    /**
     * Update particle animations
     */
    updateParticles(delta) {
        for (let i = this.particles.length - 1; i >= 0; i--) {
            const particle = this.particles[i];

            // Update position
            particle.position.add(particle.velocity);

            // Apply gravity
            particle.velocity.y -= 0.01;

            // Fade out
            particle.life -= delta;
            particle.material.opacity = particle.life;

            // Remove dead particles
            if (particle.life <= 0) {
                this.scene.remove(particle);
                this.particles.splice(i, 1);
            }
        }
    }

    /**
     * Change visual style
     */
    setStyle(style) {
        this.style = style;

        switch (style) {
            case 'neon':
                this.applyNeonStyle();
                break;
            case 'classic':
                this.applyClassicStyle();
                break;
            case 'particles':
                this.applyParticlesStyle();
                break;
        }
    }

    /**
     * Apply neon/synthwave style
     */
    applyNeonStyle() {
        this.scene.background = new THREE.Color(0x000000);
        this.gridHelper.visible = true;
        this.lights.pink.intensity = 2;
        this.lights.cyan.intensity = 2;

        // Update key materials
        this.pianoKeys.forEach((keyData) => {
            if (!keyData.isBlack) {
                keyData.mesh.material.color.setHex(0x111111);
                keyData.mesh.material.emissive.setHex(0x00ffff);
                keyData.mesh.material.emissiveIntensity = 0.2;
            } else {
                keyData.mesh.material.emissive.setHex(0xff00ff);
                keyData.mesh.material.emissiveIntensity = 0.2;
            }
        });
    }

    /**
     * Apply classic piano style
     */
    applyClassicStyle() {
        this.scene.background = new THREE.Color(0x1a1a1a);
        this.gridHelper.visible = false;
        this.lights.pink.intensity = 0.5;
        this.lights.cyan.intensity = 0.5;

        // Update key materials
        this.pianoKeys.forEach((keyData) => {
            if (!keyData.isBlack) {
                keyData.mesh.material.color.setHex(0xffffff);
                keyData.mesh.material.emissive.setHex(0x000000);
                keyData.mesh.material.emissiveIntensity = 0;
            } else {
                keyData.mesh.material.color.setHex(0x000000);
                keyData.mesh.material.emissive.setHex(0x000000);
                keyData.mesh.material.emissiveIntensity = 0;
            }
        });
    }

    /**
     * Apply particles style
     */
    applyParticlesStyle() {
        this.scene.background = new THREE.Color(0x000510);
        this.gridHelper.visible = true;
        this.gridHelper.material.opacity = 0.1;
        this.lights.pink.intensity = 1.5;
        this.lights.cyan.intensity = 1.5;

        // Make piano semi-transparent
        this.pianoKeys.forEach((keyData) => {
            keyData.mesh.material.transparent = true;
            keyData.mesh.material.opacity = 0.7;
            if (!keyData.isBlack) {
                keyData.mesh.material.color.setHex(0x333333);
            }
        });
    }

    /**
     * Handle window resize
     */
    onWindowResize() {
        this.camera.aspect = window.innerWidth / window.innerHeight;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(window.innerWidth, window.innerHeight);
    }

    /**
     * Animation loop
     */
    animate() {
        requestAnimationFrame(() => this.animate());

        const delta = this.clock.getDelta();

        // Rotate particle system slowly
        if (this.particleSystem) {
            this.particleSystem.rotation.y += 0.001;
        }

        // Update particles
        this.updateParticles(delta);

        // Render the scene
        this.renderer.render(this.scene, this.camera);
    }
}
