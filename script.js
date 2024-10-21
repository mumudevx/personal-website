// Particle.js configuration
particlesJS.load("particles-js", "particles.json", function () {
  console.log("particles.js loaded - callback");
});

// Countdown functionality
const countdownDate = new Date("2024-12-31T23:59:59").getTime();
let originalEndTime;

function updateCountdown() {
  const now = new Date().getTime();
  const distance = countdownDate - now;

  const days = Math.floor(distance / (1000 * 60 * 60 * 24));
  const hours = Math.floor(
    (distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)
  );
  const minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((distance % (1000 * 60)) / 1000);

  document.getElementById("days").textContent = days
    .toString()
    .padStart(2, "0");
  document.getElementById("hours").textContent = hours
    .toString()
    .padStart(2, "0");
  document.getElementById("minutes").textContent = minutes
    .toString()
    .padStart(2, "0");
  document.getElementById("seconds").textContent = seconds
    .toString()
    .padStart(2, "0");

  if (distance < 0) {
    clearInterval(countdownTimer);
    document.getElementById("countdown-boxes").innerHTML =
      "<div class='time-box'><span>The Renaissance has begun!</span></div>";
  }
}

const countdownTimer = setInterval(updateCountdown, 1000);
updateCountdown(); // Initial call to avoid delay

let renaissanceMusic;

function playSculptureMusic() {
  if (renaissanceMusic) {
    renaissanceMusic.play().catch((error) => {
      console.error("Error playing audio:", error);
    });
  }
}

function initializeAudio() {
  renaissanceMusic = new Audio("path/to/renaissance_music.mp3");
  renaissanceMusic.loop = true;
}

document.addEventListener("DOMContentLoaded", initializeAudio);