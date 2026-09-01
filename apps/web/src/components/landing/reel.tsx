import { Link } from "@tanstack/react-router";
import { Volume2, VolumeX } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Reveal } from "@/components/markov/reveal";
import { Button } from "@/components/ui/button";

export function CinematicReel() {
  const sectionRef = useRef<HTMLElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [sound, setSound] = useState(false);
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const apply = () => setReduced(mq.matches);
    apply();
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  }, []);

  useEffect(() => {
    const section = sectionRef.current;
    const video = videoRef.current;
    if (!section || !video || reduced) return;

    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          void video.play().catch(() => {});
        } else {
          video.pause();
        }
      },
      { threshold: 0.4 },
    );
    io.observe(section);
    return () => io.disconnect();
  }, [reduced]);

  async function toggleSound() {
    const video = videoRef.current;
    if (!video) return;
    if (sound) {
      video.muted = true;
      setSound(false);
      return;
    }
    video.muted = false;
    video.currentTime = 0;
    try {
      await video.play();
      setSound(true);
    } catch {
      video.muted = true;
      setSound(false);
    }
  }

  return (
    <section
      ref={sectionRef}
      className="relative -mt-[4.5rem] h-svh min-h-[32rem] overflow-hidden"
    >
      {reduced ? (
        <img
          src="/images/keep-the-pile.jpg?v=trailer"
          alt="Woman sprinting through night rain. Action trailer still."
          className="absolute inset-0 h-full w-full object-cover"
        />
      ) : (
        <video
          ref={videoRef}
          className="absolute inset-0 h-full w-full cursor-pointer object-cover"
          poster="/images/keep-the-pile.jpg?v=trailer"
          muted
          playsInline
          loop
          autoPlay
          preload="auto"
          onClick={toggleSound}
        >
          <source src="/videos/keep-the-pile.mp4?v=trailer" type="video/mp4" />
        </video>
      )}
      <div className="pointer-events-none absolute inset-x-0 bottom-0 h-[28%] bg-linear-to-t from-bg via-bg/75 to-transparent" />

      <div className="relative mx-auto flex h-full max-w-6xl items-end px-5 pb-10 pt-24">
        <Reveal className="flex w-full flex-wrap items-end justify-between gap-4">
          <div className="max-w-md">
            <p className="eyebrow">Trailer</p>
            <h2 className="mt-1 text-2xl font-semibold tracking-tight md:text-3xl">Keep the pile.</h2>
          </div>
          <div className="flex flex-wrap gap-3">
            <Button asChild>
              <Link to="/book">Open the desk</Link>
            </Button>
            {!reduced ? (
              <Button
                type="button"
                variant="ghost"
                onClick={toggleSound}
                aria-pressed={sound}
                aria-label={sound ? "Mute the film" : "Play the film with sound"}
              >
                {sound ? <Volume2 className="size-4" /> : <VolumeX className="size-4" />}
                {sound ? "Playing" : "Sound on"}
              </Button>
            ) : null}
          </div>
        </Reveal>
      </div>
    </section>
  );
}
