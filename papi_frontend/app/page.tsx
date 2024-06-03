import PapiLogin from "@components/PapiLogin";

export default function Home() {
  return (
    <section className="w-full flex-center flex-col">
      <h1 className="head_text text-center">
        A super boring and not personalized page
      </h1>
      <PapiLogin />
    </section>
  );
}
