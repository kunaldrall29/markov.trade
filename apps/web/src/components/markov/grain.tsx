import { useEffect, useState } from "react";

export function Grain() {
  const [p, setP] = useState(0);

  useEffect(() => {
    const onScroll = () => {
      const h = document.documentElement.scrollHeight - window.innerHeight;
      setP(h > 0 ? Math.min(1, window.scrollY / h) : 0);
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <>
      <div className="grain" aria-hidden="true" />
      <div
        className="progress-bar"
        aria-hidden="true"
        style={{ transform: `scaleX(${p})` }}
      />
    </>
  );
}
